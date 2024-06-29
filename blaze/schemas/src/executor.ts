import { fileChangesMatcherSchema } from './matchers.js'
import { Schema, notEmptyString, strictObject } from './utils.js'

const executorKindSchema = {
    enum: ['Rust', 'Node']
} satisfies Schema

const httpAuthentication = {
    oneOf: [
        strictObject({
            properties: {
                mode: {
                    const: 'Basic'
                },
                username: notEmptyString,
                password: notEmptyString
            },
            required: ['mode', 'username', 'password']
        }),
        strictObject({
            properties: {
                mode: {
                    const: 'Digest'
                },
                username: notEmptyString,
                password: notEmptyString
            },
            required: ['mode', 'username', 'password']
        }),
        strictObject({
            properties: {
                mode: {
                    const: 'Bearer'
                },
                token: notEmptyString
            },
            required: ['mode', 'token']
        })
    ]
} satisfies Schema

const gitPlainAuthentication = strictObject({
    properties: {
        username: notEmptyString,
        password: notEmptyString
    }
})

const httpTransportProperties: Record<string, Schema> = {
    insecure: {
        type: 'boolean',
        default: false
    },
    headers: {
        type: 'object',
        patternProperties: {
            '^.+$': notEmptyString
        },
        default: {}
    }
}

const sshTransportProperties: Record<string, Schema> = {
    insecure: {
        type: 'boolean',
        default: false
    },
    fingerprints: {
        type: 'array', 
        items: {
            type: 'string',
            pattern: '^(MD5|SHA1|SHA256):.+$'
        }
    }
}

const gitOptionsProperties: Record<string, Schema> = {
    path: notEmptyString,
    branch: notEmptyString,
    rev: notEmptyString,
    tag: notEmptyString,
    kind: executorKindSchema,
    pull: {
        type: 'boolean',
        default: false
    }
}

const sshAuthentication = {
    oneOf: [
        strictObject({
            properties: {
                username: notEmptyString,
                password: notEmptyString
            },
            required: ['password']
        }),
        strictObject({
            properties: {
                key: notEmptyString,
                passphrase: notEmptyString,
                username: notEmptyString
            },
            required: ['key']
        })
    ]
} as const satisfies Schema

export const executorSchema = {
    oneOf: [
        notEmptyString,
        strictObject({
            properties: {
                url: {
                    type: 'string',
                    enum: [
                        'noop',
                        'commands',
                        'exec'
                    ].map(name => `std:${name}`)
                }
            },
            required: ['url']
        }),
        strictObject({
            properties: {
                url: {
                    type: 'string',
                    pattern: '^file://.+$'
                },
                rebuild: {
                    enum: ['Always', 'OnChanges'],
                    default: 'OnChanges'
                },
                kind: executorKindSchema,
                watch: {
                    type: 'array',
                    items: fileChangesMatcherSchema
                }
            },
            required: ['url']
        }),
        strictObject({
            properties: {
                url: {
                    type: 'string',
                    pattern: '^https?://.+$'
                },
                format: {
                    const: 'Git'
                },
                authentication: gitPlainAuthentication,
                ...httpTransportProperties,
                ...gitOptionsProperties
            },
            required: ['url', 'format'],
        }),
        strictObject({
            properties: {
                url: {
                    type: 'string',
                    pattern: '^ssh://.+$'
                },
                authentication: sshAuthentication,
                ...gitOptionsProperties,
                ...sshTransportProperties
            }
        })
    ]
} as const satisfies Schema