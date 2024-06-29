const items = [
    { 
        name: "phone", 
        render: () => {
            const span = document.createElement('span')
            span.textContent = window.atob("KzMzLjYuNDkuOTMuOTEuNTE=")
            return span
        } 
    },
    { 
        name: "email", 
        render: () => {
            const anchor = document.createElement('a')
            const email = window.atob("bnphb3UucmVuYXVkQGdtYWlsLmNvbQ==")
            anchor.href = `mailto:${email}`
            anchor.textContent = email
            return anchor
        } 
    }
]

const button = document.querySelector(`#show-contact-fields`)
    
button.addEventListener('click', () => {
    
    for (const { name, render } of items){
        const placeholder = document.querySelector(`#${name}-placeholder`)
        placeholder.parentNode.replaceChild(render(), placeholder)
    }

    button.remove()
})


    