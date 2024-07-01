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
                version: '0.2.11',
                dryRun: false
            },
            runArgs: ['version'], 
            tests: null
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