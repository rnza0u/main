#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { readFile } from 'node:fs/promises'
import { join } from 'node:path'

const registry = 'registry.rnzaou.me'

async function go(){
    try {
        const json = await readFile(join(process.env['HOME'], '.docker/config.json'))
        const { auths = {} } = JSON.parse(json)
        if (registry in auths){
            console.log(`already logged in to ${registry}`)
            return
        }

    } catch (err){
        if (err.code !== 'ENOENT')
            throw err
    }

    const username = process.env['DOCKER_REGISTRY_USERNAME']
    const password = process.env['DOCKER_REGISTRY_PASSWORD']

    if (!username || !password)
        throw Error('credentials required')

    await new Promise((resolve, reject) => {
        const loginProcess = spawn('docker', [
            'login',
            '--username',
            username,
            '--password-stdin',
            registry
        ])
        loginProcess.once('error', err => reject(err))
        loginProcess.once('exit', status => {
            if (status !== 0){
                reject(Error('authentication failed'))
                return
            }
            resolve()
        })

        loginProcess.stdout.pipe(process.stdout)
        loginProcess.stderr.pipe(process.stderr)
        loginProcess.stdin.end(password)
    })
}

go().catch(err => {
    console.error(err)
    process.exit(1)
})
