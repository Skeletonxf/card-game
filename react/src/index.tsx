import React from 'react'
import { createRoot } from 'react-dom/client'
import { Provider } from "react-redux"

import App from './App'
import { store } from './store/store'

const container = document.getElementById('root')
const root = createRoot(container!)
/**
 * Tells React what to render and where to render it.
 *
 * In our case, we're rending our root `App` component to the DOM element with
 * the id of `root` in the `public/index.html` file.
 */
root.render(
    <Provider store={store}>
        <App />
    </Provider>
)
