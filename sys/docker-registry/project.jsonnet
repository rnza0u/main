{
    targets: {
        deploy: {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'compose',
                            'up',
                            '-d'
                        ]
                    }
                ],
            }
        },
        authenticate: {
            executor: 'std:exec',
            options: {
                program: 'authenticate.mjs'
            }
        },
        logout: {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: ['logout', 'registry.rnzaou.me']
                    }
                ],
            }
        },
        'collect-garbage': {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'compose', 
                            'exec', 
                            'registry', 
                            'bin/registry', 
                            'garbage-collect',
                            '/etc/docker/registry/config.yml'
                        ]
                    }
                ]
            }
        },
        'ci-setup': {
            dependencies: [
                'authenticate'
            ]
        },
        'ci-teardown': {
            dependencies: [
                'logout'
            ]
        }
    }
}