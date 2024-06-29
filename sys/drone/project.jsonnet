local docker = import 'docker.libsonnet';

{
    targets: {
        deploy: docker.composeUp()
    }
}