import { registerPlugin } from './@capacitor/core.js';

const RateApp = registerPlugin('RateApp', {
    web: () => import('./common/web-903e3ad9.js').then(m => new m.RateAppWeb()),
});

export { RateApp };
