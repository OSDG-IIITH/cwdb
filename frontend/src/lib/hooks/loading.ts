type setbool = (value: boolean) => void;

export function makeloadingstate(setloading: setbool, setshowloading: setbool, delay = 250) {
	let timer: ReturnType<typeof setTimeout> | null = null;

	function start() {
		setloading(true);
		setshowloading(false);
		if (timer) clearTimeout(timer);
		timer = setTimeout(() => {
			setshowloading(true);
		}, delay);
	}

	function stop() {
		setloading(false);
		setshowloading(false);
		if (timer) {
			clearTimeout(timer);
			timer = null;
		}
	}

	function destroy() {
		if (timer) {
			clearTimeout(timer);
			timer = null;
		}
	}

	return { start, stop, destroy };
}
