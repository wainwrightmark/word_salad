import { registerPlugin } from '../@capacitor/core.js';

const CapacitorGameConnect = registerPlugin('CapacitorGameConnect', {
    web: () => import('../common/web-aa64eed2.js').then(m => new m.CapacitorGameConnectWeb()),
});

export { CapacitorGameConnect };
