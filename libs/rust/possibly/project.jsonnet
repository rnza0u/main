local cargo = (import 'cargo.libsonnet')();
local executors = import 'executors.libsonnet';

{
    targets: cargo.all() + {
        publish: {
            executor: executors.cargoPublish()
        }
    }
}