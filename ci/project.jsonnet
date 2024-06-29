local imageRepository = 'ci';
local docker = import 'docker.libsonnet';
local cargo = (import 'cargo.libsonnet')();

local blaze = std.extVar('blaze');

local cargoTargets = cargo.all();

{
    targets: cargoTargets + {
        'build-drone': {
            executor: 'std:commands',
            description: 'Build Drone CI manifest file (.drone.yml).',
            cache: {
                invalidateWhen: {
                    filesMissing: [
                        '{{ root }}/.drone.yml'
                    ],
                    inputChanges: [
                        '.drone.jsonnet'
                    ]
                }
            },
            options: {
                commands: [
                    {
                        program: 'drone',
                        arguments: [
                            'jsonnet', 
                            '--source', '{{ project.root }}/.drone.jsonnet', 
                            '--format', 
                            '--stream', 
                            '--target', '{{ root }}/.drone.yml'
                        ]
                    },
                    {
                        program: 'drone',
                        arguments: [
                            'sign',
                            '--save',
                            'Hakhenaton/main'
                        ],
                        cwd: '{{ root }}'
                    }
                ]
            }
        },
        'build-bin': {
            executor: 'std:commands',
            description: 'Build the main CI program.',
            cache: {
                invalidateWhen: {
                    outputChanges: [
                        'target/x86_64-unknown-linux-musl/release/ci'
                    ]
                }
            },
            options: {
                commands: [
                    {
                        program: 'cross',
                        arguments: ['+nightly', 'build', '--target', 'x86_64-unknown-linux-musl', '--release']
                    }
                ]
            },
            dependencies: [
                'source'
            ]
        },
        'build-image': docker.build(imageRepository, 'registry.rnzaou.me', ['conf/**', 'Cross.toml']) + {
            description: 'Build the CI image Docker image.',
            dependencies: [
                'build-bin'
            ]
        },
        'deploy-image': docker.push(imageRepository) + {
            description: 'Deploy the CI main image to the registry.',
            dependencies: [
                'docker-registry:authenticate',
                'build-image'
            ]
        },
        'run': {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'run',
                            '--volume',
                            '/var/run/docker.sock:/var/run/docker.sock',
                            '--volume',
                            '{{ root }}:/workspace:rw',
                            '--env',
                            'OPTIONS=' + std.manifestJsonMinified(blaze.vars.ci.runOptions),
                            'registry.rnzaou.me/ci:latest',
                        ]
                    }
                ]
            },
            dependencies: [
                'build-image'
            ]
        }
    }
}