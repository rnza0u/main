import Layout from '@theme/Layout'
import styles from './about.module.css'

export default function About(){
    return <Layout
        title={'About'}
        description="Some information about the Blaze project.">
        <main className={styles.main} role="main">
            <article className={styles.about}>
                <a href="/" className={styles['back-anchor']}>
                    <i className="w-icon-left margin-right--xs"></i>Go back
                </a>
                <h1>About</h1>
                <p>
                    It is quite noticeable that our development tooling stack has become increasingly complex in recent years, 
                    especially in web development.
                </p>
                <p>More than ever, it is so important to have a way to make all these technologies work together for the sole purpose of being productive.</p>
                <p>
                    There are [many monorepo tools](https://monorepo.tools/) we could use then, however i thought they were either too complex or too tied to one or another specific technology ecosystem.
                </p>
                <p>
                    I wrote Blaze for my personal usage because i really wanted to have a generic and simple monorepo tool that i could re-use across my projects.
                    I hope you like it and find it as useful as i do !
                </p>
                <p>
                    <i>[The author](https://rnzaou.me)</i>
                </p>
            </article>
        </main>
    </Layout>
}