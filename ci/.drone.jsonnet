local secrets = [
    'DOCKER_REGISTRY_USERNAME',
    'DOCKER_REGISTRY_PASSWORD'
];

local workspacePath = '/drone/src';
local cachePath = '/tmp/cache';

local ci = {
    kind: 'pipeline',
    type: 'docker',
    name: 'Main CI pipeline',
    workspace: {
        path: workspacePath
    },
    steps: [
        {
            name: 'ci',
            image: 'registry.rnzaou.me/ci:latest',
            pull: true,
            environment: {
                ROOT: workspacePath,
                PARALLELISM: '2',
                TARGETS: std.join(",", [
                    'ci-setup',
                    'ci-build',
                    'ci-test',
                    'ci-teardown'
                ]),
                SELECTOR: std.manifestJsonMinified("All"),
                VARIABLES: std.manifestJsonMinified({
                    lint: {
                        fix: false
                    }
                }),
                CACHE_ENABLED: 'true',
                CACHE_KEY: 'ci-${DRONE_BRANCH}',
                CACHE_STORAGE: cachePath,
                CACHE_EXTRA_DIRS: std.manifestJsonMinified([
                    '/root/',
                    '.blaze/cache',
                    '.blaze/repositories',
                    '.blaze/rust'
                ])
            } + std.foldl(
                function(vars, secret) vars + {
                        [secret]: {
                            from_secret: secret 
                        }
                    }, 
                    secrets, 
                    {}
                ),
            volumes: [
                {
                    name: 'cache',
                    path: cachePath
                },
                {
                    name: 'docker-socket',
                    path: '/var/run/docker.sock'
                },
                {
                    name: 'blaze-builds',
                    path: '/var/lib/blaze/builds'
                }
            ]
        }
    ],
    volumes: [
        {
            name: 'cache',
            host: {
                path: '/var/lib/cache'
            },
        },
        {
            name: 'blaze-builds',
            host: {
                path: '/var/lib/blaze/builds'
            }
        },
        {
            name: 'docker-socket',
            host: {
                path: '/run/user/1002/docker.sock'
            }
        }
    ],
    image_pull_secrets: ['DOCKER_REGISTRY_AUTHENTICATION_JSON']
};

[
    ci,
]