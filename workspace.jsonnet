local tags = {
  rust: 'rust',
  typescript: 'typescript',
  blaze: 'blaze',
  docs: 'docs',
  lib: 'lib',
  tests: 'tests',
  web: 'web',
  node: 'node'
};

local blazeProjects = {
  'blaze-cli': {
    path: 'blaze/cli',
    description: 'Blaze command line interface crate.',
    tags: [
      tags.rust, 
      tags.blaze
    ]
  }, 
  'blaze-cli-docs': {
    path: 'blaze/cli-docs',
    description: 'Auto generated documentation for Blaze CLI (man files and mdx files for the website)',
    tags: [tags.rust, tags.blaze, tags.docs]
  },
  'blaze-core': {
    path: 'blaze/core',
    description: 'Blaze main library crate.',
    tags: [tags.rust, tags.blaze]
  },
  'blaze-tests': {
    path: 'blaze/tests',
    description: 'Blaze integration tests suite.',
    tags: [tags.rust, tags.blaze, tags.tests]
  },
  'blaze-common': {
    path: 'blaze/common',
    description: 'Blaze shared data structures.',
    tags: [tags.rust, tags.blaze]
  },
  'blaze-node-bridge': {
    path: 'blaze/node/bridge',
    description: 'Blaze Node.js executors bridge script.',
    tags: [tags.node, tags.typescript, tags.blaze]
  },
  'blaze-node-devkit': {
    path: 'blaze/node/devkit',
    description: 'Blaze Node.js executors devkit (library).',
    tags: [tags.node, tags.typescript, tags.blaze]
  },
  'blaze-rust-bridge': {
    path: 'blaze/rust/bridge',
    description: 'Blaze Rust executors bridge executable.',
    tags: [tags.rust, tags.blaze]
  },
  'blaze-rust-devkit': {
    path: 'blaze/rust/devkit',
    description: 'Blaze Rust executors devkit (library).',
    tags: [tags.rust, tags.blaze]
  },
  'blaze-website': {
    path: 'blaze/website',
    description: 'Blaze main documentation website.',
    tags: [tags.web, tags.docs, tags.node]
  },
  'blaze-assets': {
    path: 'blaze/assets',
    description: 'Blaze brand assets.',
    tags: [tags.blaze]
  },
  'blaze-schemas': {
    path: 'blaze/schemas',
    description: 'Blaze JSON schemas.',
    tags: [tags.blaze, tags.docs, tags.node]
  },
  'blaze-downloads': {
    path: 'blaze/downloads',
    description: 'Blaze downloads REST API',
    tags: [tags.blaze, tags.web, tags.rust]
  }
};

local rustLibs = {
  'hash-value-rs': {
    path: 'libs/rust/hash-value',
    description: 'Serde value library that provides the Hash trait.',
    tags: [tags.rust, tags.lib]
  },
  'possibly-rs': {
    path: 'libs/rust/possibly',
    description: 'matches!() like macro, but returning an Option<T>.',
    tags: [tags.rust, tags.lib]
  }
};

local jsLibs = {
  'eslint-config': {
    path: 'libs/js/eslint-config',
    tags: [tags.typescript]
  }
};

local sys = {
  'cross-builders': 'sys/cross-builders',
  'docker-registry': 'sys/docker-registry',
  'reverse-proxy': 'sys/reverse-proxy',
  'drone': 'sys/drone'
};

local nodeExecutors = {
  [name]: {
    path: 'executors/' + name,
    tags: [tags.node, tags.typescript]
  } for name in [
    'cargo-version-check',
    'npm-version-check',
    'push-tags',
    'cargo-publish',
    'npm-publish',
    'package-binaries'
  ]
} + {
  'node-executors-common': {
    path: 'executors/node-common',
    tags: [tags.node, tags.typescript]
  }
};

local personal = {
  'cv': {
    path: 'personal/cv',
    description: 'My personal website',
    tags: [tags.node, tags.typescript, tags.web]
  },
  'playground': 'personal/playground'
};

local blazeSelector = std.objectFields(blazeProjects);

{
  name: 'rnz',
  projects: blazeProjects + rustLibs + sys + nodeExecutors + jsLibs + personal + {
    ci: {
      path: 'ci',
      description: 'Main continuous integration files.'
    },
  },
  settings: {
    defaultSelector: blazeSelector,
    selectors: {
      blaze: blazeSelector
    },
    logLevel: 'Warn',
    parallelism: 'Infinite'
  }
}
