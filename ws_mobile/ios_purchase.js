document.addEventListener('deviceready', onDeviceReady);

const { store, ProductType, Platform } = CdvPurchase;

const android_products = [
  {
    type: ProductType.NON_CONSUMABLE,
    id: 'removeads',
    platform: Platform.APPLE_APPSTORE,
  },

  {
    type: ProductType.NON_CONSUMABLE,
    id: 'geographypack',
    platform: Platform.APPLE_APPSTORE,
  }, {
    type: ProductType.NON_CONSUMABLE,
    id: 'naturalworldpack',
    platform: Platform.APPLE_APPSTORE,
  }, {
    type: ProductType.NON_CONSUMABLE,
    id: 'ussports',
    platform: Platform.APPLE_APPSTORE,
  },


  {
    type: ProductType.CONSUMABLE,
    id: 'hints500',
    platform: Platform.APPLE_APPSTORE,
  }, {
    type: ProductType.CONSUMABLE,
    id: 'hints50',
    platform: Platform.APPLE_APPSTORE,
  }, {
    type: ProductType.CONSUMABLE,
    id: 'hints100',
    platform: Platform.APPLE_APPSTORE,
  }, {
    type: ProductType.CONSUMABLE,
    id: 'hints25',
    platform: Platform.APPLE_APPSTORE,
  },
]


function onDeviceReady() {

  console.log("Purchases OnDeviceReady");

  // refreshUI();
  store.register(android_products);
  store.when()
    .productUpdated(refreshUI)
    .approved(finishPurchase);
  store.initialize([Platform.GOOGLE_PLAY]);
}

function finishPurchase(transaction) {
  console.log(`Purchase Plugin: transaction complete ${transaction}`);
  transaction.finish();
  refreshUI();
}


function refreshUI() {
}

export async function get_products() {
  return store.products;
}

export async function refresh_and_get_products() {
  await store.update();

  return store.products;
}

export async function purchase_product(options) {
  const product = store.get(options.id);
  const offer = product.getOffer();

  console.log(`Request to purchase "${options.id}" "${offer.id}"`);

  var result = await store.order(offer);

  if (result == undefined) {
    console.info("Purchase Succeeded");
    return {
      purchased: true
    };
  }
  else {
    console.error("Purchase Failed");
    console.error(`${result}`);
    return {
      purchased: false
    }
  }

}