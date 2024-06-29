local defaultRegistry = 'registry.rnzaou.me';

{
    build: function(repository, registry=defaultRegistry, extraFiles=[]){
        executor: 'std:commands',
        cache: {
            invalidateWhen: {
                inputChanges: ['Dockerfile'] + extraFiles
            }
        },
        options: {
            commands: [
                {
                    program: 'docker',
                    arguments: ['build', '-t', registry + '/' + repository, '.']
                }
            ]
        },
    },
    push: function(repository, registry=defaultRegistry){
        executor: 'std:commands',
        options: {
            commands: [
                {
                    program: 'docker',
                    arguments: [
                        'push',
                        registry + '/' + repository
                    ]
                }
            ]
        }
    },
    composeUp: function() {
        executor: 'std:commands',
        options: {
            commands: [
                {
                    program: 'docker',
                    arguments: [
                        'compose',
                        'up',
                        '--detach',
                        '--remove-orphans',
                        '--pull', 
                        'always'
                    ]
                }
            ]
        }
    }
}