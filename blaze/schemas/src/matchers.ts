import { Schema, notEmptyString, strictObject } from './utils.js'

export const fileChangesMatcherSchema = {
    oneOf: [
        notEmptyString,
        strictObject({
            properties: {
                pattern: notEmptyString,
                exclude: {
                    type: 'array',
                    items: notEmptyString,
                    default: []
                },
                root: notEmptyString,
                behavior: {
                    enum: [
                        'Mixed',
                        'Timestamps',
                        'Hash'
                    ],
                    default: 'Mixed'
                }
            },
            required: ['pattern']
        })
    ]
} satisfies Schema