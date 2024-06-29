local targets = import '../targets.jsonnet';
local LocalEnv = import '../core/local-env.jsonnet';
local cargo = (import 'cargo.libsonnet')('nightly', ['-Z', 'bindeps']);

local cargoTargets = cargo.all({
    workspaceDependencies: ['blaze-cli']
});

{
    targets: cargoTargets + {
        build: {
            executor: 'std:commands',
            description: 'Build the documentation files.',
            options: {
                commands: [
                    {
                        program: 'cargo',
                        arguments: ['+nightly', 'run', '-Z', 'bindeps', '--release'],
                        environment: LocalEnv(targets.release) + {
                            OUT_DIR: '{{ project.root }}/dist'
                        }
                    }
                ],
            },
            cache: {
                invalidateWhen: {
                    outputChanges: ['dist/**']
                }
            },
            dependencies: [
                'source'
            ]
        },
        'ci-build': {
            dependencies: ['lint', 'check']
        }
    }
}