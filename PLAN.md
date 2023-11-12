# ci

Idea: Everything to do with my personal CI/CD stack

Responsibilities:
- Managing git repos
- Notifying of git/build/deploy actions
- Reading git changesets, determining targets to run

3 components:
- `bot`: user-facing input/reporting interface (discord bot)
- `tool`: operations relevant while running buildkite jobs

Example input:
`flake.nix`
```nix
{
    inputs = { ... };
    outputs = ({ ... }: {
        ciActions = {
            deployBot = {
                # This operation is only triggered on push if ALL these
                # conditions are true.
                # Condition strings are regular expressions that must match the
                # entirety of the string
                conditions = [
                    # Must start with v
                    { tag = "v.*"; }
                    # Must be exactly "master"
                    { branch = "master"; }
                    # self.packages.bot must have changed during this set
                    changed = [ self.packages.bot ];
                    # Other examples
                    # Must be on a branch starting with "release/"
                    # { branch = "release/.*"; }
                    # Must be on a tag of the format "release/../.."
                    # { tag = "release/[^/]*/[^/]"; }
                ];
                # Run synchronously before `builds` and `steps`
                # Idea: Do cheaper metadata/linter checks before building full
                # application (not sure if there's a great way to do this with
                # nix)
                checks = [
                    {
                        name = "lint-bot";
                        displayName = "Lint bot";
                        target = self.apps.check-lint;
                    }
                ];
                # Built synchronously before `checks` or `steps` are run.
                # If there's no shared cache between these builds and the
                # steps, these will only really serve as a pre-deploy step to
                # make sure these will build.
                # (Not required here, should be discovered)
                # builds = [
                #     self.packages.bot
                #     self.packages.deploy-bot
                #     self.packages.tool
                #     self.apps.deploy-bot-staging
                #     self.apps.deploy-bot-production
                # ];
                # Idea: A set of standardised steps that map to
                # buildkite steps, that get used to generate
                # steps a set of steps for arbitrary actions
                deploySteps = [
                    {
                        name = "deploy enabled";
                        target = self.apps.check-deploy-enable;
                    }
                    { type = "wait"; }
                    {
                        type = "run";
                        label = "CI Bot: staging deploy";
                        name = "deploy-bot-staging";
                        target = self.apps.deploy-bot-staging;
                    }
                    # Requires manual human input
                    { type = "block"; }
                    {
                        type = "run";
                        label = "CI Bot: production deploy";
                        name = "deploy-bot-production";
                        target = self.apps.deploy-bot-production;
                    }
                ];
            };
        };
    });
}
```

<!--
NOTE: Need to make sure runtime environment variable interpolation works here.
        Only environment variables actually _set_ by Buildkite can be
        interpolated when uploading pipelines, and we have to encode
        differently to interpolate at runtime
See:
- https://buildkite.com/docs/pipelines/environment-variables#variable-interpolation
- https://buildkite.com/docs/pipelines/environment-variables#runtime-variable-interpolation
-->
A tag push to master translates to something like:
```yaml
env:
    # For plugin/webhook-processing purposes
    DENBEIGH_MANAGED: "true"
    # Only becomes "true" when we trigger the "deploy" pipeline
    DENBEIGH_DEPLOY: "false"

    BUILDKITE_PLUGIN_CICD_OPERATION: "check"
    # Required so we know which pipeline to trigger
    BUILDKITE_PLUGIN_CICD_DEPLOY_PIPELINE_NAME: "deploy"
    BUILDKITE_PLUGIN_CICD_REPO: "$BUILDKITE_REPO"
    # Used below
    DEPLOY_TOOL_VERSION: "v0.0.1"
    TOOL_TARGET: "github:denbeigh2000/ci/$${DEPLOY_TOOL_VERSION}#apps.tool"

steps:
    # This first step just uploads dynamically-generated steps...
    - command: >-
        nix run $${TOOL_TARGET} plan
      label: "Plan pipeline"
    # ...

    - plugins:
        - denbeigh2000/cicd:
            repo: '$BUILDKITE_REPO'
      # Checks (because we pushed a tag)
      command: >-
        nix run $${TOOL_TARGET} check
            --target .#apps.check-lint
      label: "Check: Lint bot"
      key: "check-lint"
    - plugins:
        - denbeigh2000/cicd:
            repo: '$BUILDKITE_REPO'
      command: >-
        nix run $${TOOL_TARGET} check
            --target .#apps.check-deploy-enable
      label: "Check: deploy enabled"
      key: "check-deploy-enabled"

    # Wait for check completion
    - wait

    - plugins:
        - denbeigh2000/cicd:
            repo: '$BUILDKITE_REPO'
      # Builds (automatically discovered)
      # NOTE: will need to introduce some kind of filtering over this
      #   (eventually).
      # will also need to use --keep-going within our build invoker so that we
      # can build all possible successful builds
      command: >-
        nix run $${TOOL_TARGET} build --
            .#packages.bot
            .#packages.deploy-bot
            .#packages.tool
            .#packages.apps.deploy-bot-staging
            .#packages.apps.deploy-bot-production
      label: "Build packages"
      key: "build"

    # Wait for build completion
    - wait

    # Deployment (because we pushed a tag)
    - plugins:
        - denbeigh2000/cicd:
            repo: '$BUILDKITE_REPO'
      trigger: "deploy"
      async: true
      label: ':rocket: Trigger deploy'
      build:
        message: 'Deploy from $BUILDKITE_PIPELINE_NAME#$BUILDKITE_BUILD_NUMBER'
        commit: '$BUILDKITE_COMMIT'
        branch: '$BUILDKITE_BRANCH'
        env:
            # (where this is the JSON representation of
            # .#outputs.ciActions.deployBot.deploySteps
            STEP_DATA: "{\"steps\":[...]}"
            BUILDKITE_PLUGIN_CICD_REPO: '$BUILDKITE_REPO'
            DENBEIGH_MANAGED: "true"
            DENBEIGH_DEPLOY: "true"
            DENBEIGH_DEPLOY_ACTION_NAME: "deployBot"
        meta_data:
            # There will be similar metadata
            "denbeighDeploy": true
```

And that should trigger another pipeline that ends up populated to something
like:
```yaml
env:
    BUILDKITE_PLUGIN_CICD_OPERATION: "deploy"
    DEPLOY_TOOL_VERSION: "v0.0.1"
    TOOL_TARGET: "github:denbeigh2000/ci/$${DEPLOY_TOOL_VERSION}#apps.tool"

steps:
    # Plan our pipeline
    - plugins:
        - thedyrt/skip-checkout#v0.1.1: ~
      command: >-
        nix run $$TOOL_TARGET plan
      label: "Plan"
      key: "plan"

    - plugins:
        - denbeigh2000/cicd: ~
      command: >-
        nix run $$TOOL_TARGET deploy
            --target .#apps.check-deploy-enable
      label: "CI Bot: staging deploy"

    # Run our staging deployment (step 1)
    - plugins:
        - denbeigh2000/cicd: ~
      command: >-
        nix run $$TOOL_TARGET deploy
            --target .#apps.deploy-bot-staging
      label: "CI Bot: staging deploy"

    # Block before permitting production deployment (step 2)
    - block

    # Run our production deployment (step 3)
    - plugins:
        - denbeigh2000/cicd: ~
      command: >-
        nix run $$TOOL_TARGET deploy
            --target .#apps.deploy-bot-production
      label: "CI Bot: production deploy"
```

While this is all happening, our worker will receive buildkite events, and
through the presence of DENBEIGH_MANAGED/DENBEIGH_DEPLOY env vars, the bot
should be able to provide more detailed and contextual embeds for
builds/deploys

Potential wants for 2.0 bot embeds:
- row + status per tracked derivation (build)
- row + status per tracked build step (deploy)

2.0 embeds could be something like:
```
Denbeigh Stevens
:x: .dotfiles (#6969)

### Details:
    `abc123` (smith/test): Fix asdasd (<t:123:R>)

### Checks:
 - :x: Lint bot

### Builds:
 - :check: `bot-1.0.0`
 - :check: 'deploy-bot-1.0.0'
 - :x:     'ci-tool-1.0.1'
...
```

With this all laid out, what pieces of functionality do we need from our
pipeline tool?

`tool plan`
Given the local context of:
    - a Buildkite job 
    - the checked out local repo (well-known, local file state)
    - local environment variables
Create (and maybe upload?) a buildkite YAML pipeline definition for this job.

`tool check --target [LOCAL_DERIVATION]`
Runs the given Nix command as a CI check. Main purpose for this vs. just
running alone is to record rich failure information in metadata.

`tool build ...`
Builds all the given targets in this commit.
Coalesced into one command so we can avoid over-building.

This command should build with `--keep-going`, and track the derivations that
_were_ successfully built, so that we can do rich reporting of which
derivations built and which failed to build.

`tool deploy --target [LOCAL_APP]`
Runs the given targets for deployment.
