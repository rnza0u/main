{
    vars: {
        lint: {
            fix: true
        },
        docker: {
            registry: 'registry.rnzaou.me'
        },
        blaze: {
            publish: {
                version: '0.2.10',
                dryRun: false
            },
            runArgs: ['version'], 
            tests: null
        },
        ci: {
            runOptions: {
                parallelism: 'Infinite',
                root: '/workspace',
                branch: 'master',
                targets: [ 
                    'ci-build'
                ]
            }
        },
        sudo: false
    },
    include: [
        {
            path: '{{ root }}/user-variables.jsonnet',
            optional: true
        }
    ]
}