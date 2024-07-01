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
            tests: null,
            rust: {
                channel: 'nightly-2024-06-25'
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