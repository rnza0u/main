local docker = import 'docker.libsonnet';
local npm = import 'npm.libsonnet';

{
    targets: {
        
        install: npm.install(),
        source: npm.source(),
        'build-bundle': npm.build() + {
            description: 'Build the website files for production.'
        },
        'build-image': docker.build('cv') + {
            dependencies: [
                'build-bundle'
            ]
        },
        'deploy-image': docker.push('cv') + {
            dependencies: [
                'docker-registry:authenticate',
                'build-image'
            ]
        },
        'serve-parcel': {
            executor: 'std:commands',
            description: 'Serve in dev mode with parcel.',
            options: {
                commands: [
                    {
                        program: './node_modules/.bin/parcel'
                    }
                ]
            },
            dependencies: ['install']
        },
        'serve-compose': {
            executor: 'std:commands',
            description: 'Build for production and serve locally (for testing purpose only).',
            options: {
                commands: [
                    {
                        program: 'docker',
                        arguments: [
                            'compose', 
                            '-f', 
                            'docker-compose-dev.yml',
                            'up', 
                            '--pull', 
                            'never', 
                            '--force-recreate'
                        ]
                    }
                ]
            },
            dependencies: [
                'build-image'
            ]
        },
        deploy: docker.composeUp() + {
            description: 'Deploy in production using docker compose.'
        },
        clean: npm.clean({
            extraDirectories: ['.parcel-cache']
        }),
        'ci-build': {
            dependencies: ['build-image']
        },
        publish: {
            dependencies: ['deploy-image']
        }
    }
}