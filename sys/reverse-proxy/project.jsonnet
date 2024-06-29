local docker = import 'docker.libsonnet';

{
    targets: {
        build: docker.build(
            'reverse-proxy', 
            'registry.rnzaou.me', 
            [
                { pattern: 'conf.d/*', exclude: ['project.jsonnet'] },
                'nginx.conf'
            ]
        ),
        push: docker.push('reverse-proxy') + {
            dependencies: [
                'docker-registry:authenticate',
                'build'
            ]
        },
        'renew-certificates': {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'compose',
                            'exec',
                            '-t',
                            'reverse-proxy',
                            '/scripts/run_certbot.sh',
                            'force'
                        ]
                    }
                ]
            }
        },
        deploy: docker.composeUp()
    }
}