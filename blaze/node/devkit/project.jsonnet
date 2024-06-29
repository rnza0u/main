local npm = import 'npm.libsonnet';
local executors = import 'executors.libsonnet';
local blaze = std.extVar('blaze');

{
    targets: npm.all('@blaze-repo/node-devkit') + {
        publish: {
            executor: executors.npmPublish(),
            options: {
                dryRun: blaze.vars.blaze.publish.dryRun
            },
            dependencies: [
                'build',
                'version-check'
            ]
        },
        'version-check': {
            executor: executors.npmVersionCheck(),
            options: {
                version: blaze.vars.blaze.publish.version
            }
        },
        'ci-build': {
            dependencies: ['lint', 'build']
        }
    }
}