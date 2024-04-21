// uno.config.ts
import { defineConfig, presetUno } from 'unocss'
import presetIcons from '@unocss/preset-icons'

export default defineConfig({
	presets: [
		presetUno({
			/* options */
		}),
		presetIcons({
			/* options */
		}),
	],
	// ...UnoCSS options
})
