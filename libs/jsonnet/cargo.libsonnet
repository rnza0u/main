function (channel = 'stable', extraArgs = [])

local blaze = std.extVar('blaze');

local args = function (mainArgs) (if channel != 'stable' then ['+' + channel] else [])
    + mainArgs
    + extraArgs;

{
    source(workspaceDependencies=[], extraSourceDirectories=[]): {
        cache: {
            invalidateWhen: {
                inputChanges: [
                    'src/**',
                    'Cargo.toml'
                ] + extraSourceDirectories
            }
        },
        dependencies: std.map(function(dep) dep + ':source', workspaceDependencies)
    },
    clean(extraTargetDirs=[]): {
        executor: 'std:commands',
        description: 'Clean all cargo files.',
        options: {
            commands: [
                {
                    program: 'cargo',
                    arguments: args(['clean'])
                }
            ] + std.map(
                function (dir){ 
                    program: 'cargo', 
                    arguments: args(['clean', '--target-dir', dir]) 
                }, 
                extraTargetDirs
            )
        }
    },
    check(): {
        executor: 'std:commands',
        description: 'Check source code.',
        cache: {},
        options: {
            commands: [
                {
                    program: 'cargo',
                    arguments: args(['check'])
                }
            ]
        },
        dependencies: [
            'source'
        ]
    },
    lint(): {
        executor: 'std:commands',
        description: 'Formats and check source files.',
        cache: {},
        options: {
            commands: [
                {
                    program: 'cargo',
                    arguments: (if channel != 'stable' then ['+' + channel] else []) 
                        + ['fmt']
                        + (if blaze.vars.lint.fix then [] else ['--check'])
                },
                {
                    program: 'cargo',
                    arguments: args(['clippy', '--no-deps'])
                }
            ]
        },
        dependencies: [
            'source'
        ]
    },
    local api = self,
    all(options = {
        workspaceDependencies: [],
        extraSourceDirectories: [],
        extraTargetDirs: []
    }): {
        source: api.source(
            if std.objectHas(options, 'workspaceDependencies') then options.workspaceDependencies else [],
            if std.objectHas(options, 'extraSourceDirectories') then options.extraSourceDirectories else []
        ),
        lint: api.lint(),
        check: api.check(),
        clean: api.clean(if std.objectHas(options, 'extraTargetDirs') then options.extraTargetDirs else [])
    }
}