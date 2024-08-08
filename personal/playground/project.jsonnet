{
    targets: {
        'rust-executor': {
            executor: {
                url: 'ssh://git@github.com/rnza0u/blaze-test-rust-executor.git',
                branch: 'master',
                fingerprints: [
                    'SHA256:uNiVztksCsDhcc0u9e8BujQXVUpKZIDTMczCvj3tD2s',
                    'SHA256:p2QAMXNIC1TJYWeIOttrVc98/R1BUFWu3/LiyKgUfQM',
                    'SHA256:+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU'
                ]/*,
                authentication: {
                    key: '{{ environment.HOME }}/.ssh/ed25519.pem',
                    passphrase: '{{#if environment.SSH_PRIVATE_KEY_PASSPHRASE }}{{ environment.SSH_PRIVATE_KEY_PASSPHRASE }}{{/if}}'
                }*/
            }
        },
        'shell': {
            executor: 'std:commands',
            options: {
                commands: [
                    {
                        program: 'echo',
                        arguments: [
                            '{{ shell "echo $SHELL" shell="/bin/sh" shellKind="posix" trim=true }}'
                        ]
                    }
                ]
            }
        }
    }
}