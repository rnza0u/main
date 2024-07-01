local ci = {
  kind: 'pipeline',
  type: 'docker',
  name: 'Main CI pipeline',
  workspace: {
    path: '/drone/src',
  },
  steps: [
    {
      name: 'ci',
      image: 'registry.rnzaou.me/ci:latest',
      pull: true,
      environment: {
        CACHE_ENABLED: '${CACHE_ENABLED:-true}',
        CACHE_IGNORE_EXISTING: '${CACHE_IGNORE_EXISTING:-false}',
        CACHE_STORAGE: '/var/lib/cache',
        CACHE_KEY: 'ci-${DRONE_BRANCH}',
        CACHE_EXTRA_DIRS: std.join(',', [
            "/root/.cargo",
            "/root/.npm",
            "/root/.rustup",
            ".blaze/cache",
            ".blaze/repositories",
            ".blaze/rust"
        ])
      } + std.foldl(
        function(vars, secret) vars {
          [secret]: {
            from_secret: secret,
          },
        },
        [
          'DOCKER_REGISTRY_USERNAME',
          'DOCKER_REGISTRY_PASSWORD',
          'CARGO_TOKEN',
          'NPM_TOKEN'
        ],
        {}
      ),
      volumes: [
        {
          name: 'cache',
          path: '/var/lib/cache',
        },
        {
          name: 'docker-socket',
          path: '/var/run/docker.sock',
        },
        {
          name: 'blaze-builds',
          path: '/var/lib/blaze/builds',
        },
        {
          name: 'ssh',
          path: '/root/.ssh',
        },
      ],
    },
  ],
  volumes: [
    {
      name: 'ssh',
      host: {
        path: '/var/lib/drone/.ssh',
      },
    },
    {
      name: 'cache',
      host: {
        path: '/var/lib/cache',
      },
    },
    {
      name: 'blaze-builds',
      host: {
        path: '/var/lib/blaze/builds',
      },
    },
    {
      name: 'docker-socket',
      host: {
        path: '/run/user/1002/docker.sock',
      },
    },
  ],
  image_pull_secrets: ['DOCKER_REGISTRY_AUTHENTICATION_JSON'],
};

[
  ci,
]
