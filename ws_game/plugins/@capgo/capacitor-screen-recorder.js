import { registerPlugin } from '../@capacitor/core.js';

const ScreenRecorder = registerPlugin("ScreenRecorder", {
    web: () => import('../common/web-acc28c5b.js').then((m) => new m.ScreenRecorderWeb()),
});

export { ScreenRecorder };
