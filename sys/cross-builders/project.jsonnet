local sdkBuilderRepository = 'registry.rnzaou.me/osx-sdk-builder';

local osxSdkFile = 'MacOSX{{ environment.MACOS_SDK_VERSION }}.sdk.tar.bz2';

local docker = import 'docker.libsonnet';
local architectures = [
    'aarch64-apple-darwin',
    'x86_64-apple-darwin'
];

{
    targets: {
        'build-osx-sdk-builder': {
            executor: 'std:commands', 
            cache: {
                invalidateWhen: {
                    inputChanges: [
                        'osx-sdk-builder/Dockerfile',
                        'osx-sdk-builder/build.sh'
                    ]
                }
            },
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'build',
                            '-t', sdkBuilderRepository,
                            '.'
                        ],
                        cwd: '{{ project.root }}/osx-sdk-builder'
                    }
                ]
            }
        },
        'build-osx-sdk': {
            executor: 'std:commands', 
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'run',
                            '-v', '{{ environment.XCODE_HOME }}/{{ environment.XCODE_ARCHIVE }}:/build/xcode.xip:ro',
                            '-v', './cross/docker/cross-toolchains/docker/:/out:rw',
                            '-e', 'SDK_FILENAME=' + osxSdkFile,
                            sdkBuilderRepository,
                        ]
                    }
                ]
            },
            cache: {
                invalidateWhen: {
                    inputChanges: [
                        {
                            pattern: '{{ environment.XCODE_ARCHIVE }}',
                            root: '{{ environment.XCODE_HOME }}',
                            behavior: 'Timestamps'
                        }
                    ],
                    filesMissing: [
                        'cross/docker/cross-toolchains/docker/' + osxSdkFile,
                    ]
                }
            },
            dependencies: [
                'clone-cross',
                'build-osx-sdk-builder'
            ]
        },
        'push-images': {
            dependencies: ['push-' + name for name in architectures]
        },
        'clone-cross': {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'rm',
                        arguments: ['-rf', 'cross']
                    },
                    {
                        program: 'git',
                        arguments: [
                            'clone',
                            'https://github.com/rnza0u/cross'
                        ]
                    },
                    {
                        program: 'git',
                        arguments: [
                            '-C',
                            '{{ project.root }}/cross',
                            'submodule',
                            'update',
                            '--init',
                            '--remote'
                        ]
                    }
                ]
            },
            cache: {
                invalidateWhen: {
                    filesMissing: [
                        'cross'
                    ]
                }
            }
        },
        clean: {
            executor: 'std:commands',
            options: {
                commands: [
                    'rm -rf osx-sdk-builder/dist cross'
                ],
                shell: true
            }
        }
    } + {
        ['push-' + name]: docker.push(name + '-cross', 'registry.rnzaou.me') + {
            dependencies: [
                'docker-registry:authenticate',
                'build-image-' + name
            ]
        } for name in architectures
    } + {
        ['build-image-' + name]: {
            executor: 'std:commands',
            cache: {},
            options: {
                commands: [
                    {
                        program: 'cargo',
                        arguments: [
                            'build-docker-image',
                            name + '-cross',
                            '--build-arg', 'MACOS_SDK_DIR=cross-toolchains/docker',
                            '--build-arg', 'MACOS_SDK_FILE=' + osxSdkFile,
                            '--repository', 'registry.rnzaou.me',
                            '--tag', 'latest'
                        ],
                        cwd: 'cross'
                    }
                ],
                shell: true
            },
            dependencies: [
                'build-osx-sdk'
            ]
        } for name in architectures
    }
}