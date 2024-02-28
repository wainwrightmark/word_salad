const { store, ProductType, Platform } = CdvPurchase;

//spellchecker:disable

const android_products = [
  {
    type: ProductType.NON_CONSUMABLE,
    id: 'removeads',
    platform: Platform.GOOGLE_PLAY,
  },

  {
    type: ProductType.NON_CONSUMABLE,
    id: 'geographypack',
    platform: Platform.GOOGLE_PLAY,
  }, {
    type: ProductType.NON_CONSUMABLE,
    id: 'naturalworldpack',
    platform: Platform.GOOGLE_PLAY,
  }, {
    type: ProductType.NON_CONSUMABLE,
    id: 'ussports',
    platform: Platform.GOOGLE_PLAY,
  },


  {
    type: ProductType.CONSUMABLE,
    id: 'hints500',
    platform: Platform.GOOGLE_PLAY,
  }, {
    type: ProductType.CONSUMABLE,
    id: 'hints50',
    platform: Platform.GOOGLE_PLAY,
  }, {
    type: ProductType.CONSUMABLE,
    id: 'hints100',
    platform: Platform.GOOGLE_PLAY,
  }, {
    type: ProductType.CONSUMABLE,
    id: 'hints25',
    platform: Platform.GOOGLE_PLAY,
  },
]

const ios_products = [
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

//spellchecker:enable

export async function initialize_and_get_products(platform, on_approved) {

  console.log("Purchases OnDeviceReady");

  if (platform == Platform.GOOGLE_PLAY) {
    store.register(android_products);
  }
  else if (platform == Platform.APPLE_APPSTORE) {
    store.register(ios_products);
  } else {
    throw new Error(`Unexpected platform "${platform}"`);
  }

  await store.initialize([platform]);

  store.when()
    //.productUpdated(refreshUI)
    .approved(function (transaction) {
      console.log(`Purchase Plugin: transaction complete ${transaction}`);
      on_approved(transaction);
      transaction.finish();
    });

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
      success: true
    };
  }
  else {
    console.error("Purchase Failed");
    console.error(`${result}`);
    return {
      success: false
    }
  }

}