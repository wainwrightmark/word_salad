document.addEventListener('deviceready', onDeviceReady);

const { store, ProductType, Platform } = CdvPurchase;

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
  console.log("Purchase Plugin: transaction complete ${transaction}");
  transaction.finish();
  refreshUI();
}


function refreshUI() {
}

export async function get_products() {
  return store.products;
}

export async function purchase_product(options){



  const product = store.get(options.id);
  const offer = product.getOffer();

  console.log(`Request to purchase "${options.id}" "${offer.id}"`);

  var result = await store.order(offer);

  if (result.isError){
    console.error(`${result}`)
    return {
      purchased: false
    }
  }else{
    return {
      purchased: true
    };
  }

}