type shortcutmap = Record<string, () => void>;

type bindopts = {
	ignoretyping?: boolean;
};

export function bindshortcuts(shortcuts: shortcutmap, opts: bindopts = {}) {
	const ignoretyping = opts.ignoretyping ?? true;
	const handler = (event: KeyboardEvent) => {
		if (ignoretyping) {
			const tag = document.activeElement?.tagName;
			if (tag === 'INPUT' || tag === 'TEXTAREA') return;
		}

		const action = shortcuts[event.key];
		if (!action) return;
		event.preventDefault();
		action();
	};

	window.addEventListener('keydown', handler);
	return () => window.removeEventListener('keydown', handler);
}

export function bindslashfocus(getinput: () => HTMLInputElement | null) {
	return bindshortcuts({
		'/': () => {
			getinput()?.focus();
		}
	});
}
