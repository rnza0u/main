local blaze = std.extVar('blaze');

{
    install(workspaceDependencies = []): {
        executor: 'std:commands',
        description: 'Install NPM dependencies.',
        options: {
            commands: std.map(function(dep){
                program: 'npm',
                arguments: ['link', '--install-links', blaze.root + '/' + blaze.workspace.projects[dep].path]
            }, workspaceDependencies) + [
                {
                    program: 'npm',
                    arguments: [if std.objectHas(blaze.environment, 'CI') && blaze.environment['CI'] == "true" then 'ci' else 'install']
                }
            ]
        },
        cache: {
            invalidateWhen: {
                filesMissing: ['node_modules'],
                outputChanges: [
                    'package-lock.json'
                ],
                inputChanges: [
                    'package.json'
                ]
            }
        },
        dependencies: std.map(function(dep) dep + ':build', workspaceDependencies)
    },
    source(extraSourceMatchers = []): {
        description: 'Project source files.',
        cache: {
            invalidateWhen: {
                inputChanges: [
                    'src/**',
                    'tsconfig*.json'
                ] + extraSourceMatchers
            }
        },
        dependencies: ['install']
    },
    build(extraArtefactMatchers = []): {
        description: 'Build the project files.',
        executor: 'std:commands',
        options: {
            commands: [
                {
                    program: 'npm',
                    arguments: [
                        'run',
                        'build',
                    ]
                }
            ]
        },
        cache: {
            invalidateWhen: {
                outputChanges: [
                    'dist/**',
                    'lib/**',
                ] + extraArtefactMatchers
            }
        },
        dependencies: ['source']
    },
    lint(): {
        local eslintConfigRoot = blaze.root + '/' + blaze.workspace.projects['eslint-config'].path,
        executor: 'std:commands',
        options: {
            commands: [
                {
                    program: eslintConfigRoot + '/node_modules/.bin/eslint',
                    arguments: [
                        '--config',
                        eslintConfigRoot + '/dist/eslint.config.js',
                    ] + (if blaze.vars.lint.fix then ['--fix'] else [])
                    + [blaze.project.root],
                    environment: {
                        ESLINT_USE_FLAT_CONFIG: 'true'
                    }
                }
            ]
        },
        cache: {},
        dependencies: [
            'source',
            'eslint-config:build'
        ]
    },
    clean(options={unlinkPackage: null, extraDirectories: []}): {
        executor: 'std:commands',
        description: 'Clean NPM dependencies and build files.',
        options: {
            commands: [
                {
                    program: 'rm',
                    arguments: [
                        '-rf',
                        'node_modules',
                        'dist',
                        'lib'
                    ] + (if std.objectHas(options, 'extraDirectories') then options.extraDirectories else [])
                },
            ] + (if std.objectHas(options, 'unlinkPackage') 
                then [
                    {
                        program: 'npm',
                        arguments: [
                            'uninstall', 
                            '--global', 
                            options.unlinkPackage
                        ]
                    }
                ] 
                else []
            )
        }
    },
    local api = self,
    all(
        packageName, 
        options = {
            workspaceDependencies: [],
            sourceExtraMatchers: [],
            artefactExtraMatchers: [],
            cleanExtraDirectories: []
        }
    ): {
        install: api.install(if std.objectHas(options, 'workspaceDependencies') then options.workspaceDependencies else []),
        source: api.source(if std.objectHas(options, 'sourceExtraMatchers') then options.sourceExtraMatchers else []),
        lint: api.lint(),
        build: api.build(if std.objectHas(options, 'artefactExtraMatchers') then options.artefactExtraMatchers else []),
        clean: api.clean({
            packageName: packageName,
            extraDirectories: if std.objectHas(options, 'cleanExtraDirectories') then options.cleanExtraDirectories else []
        }),
    }
}