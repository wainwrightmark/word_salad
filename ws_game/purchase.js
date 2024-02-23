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


  // var myProduct = store.get('geographypack', Platform.GOOGLE_PLAY);
  // const myTransaction = store.findInLocalReceipts(myProduct);

  // console.log(`Purchase Plugin: product: ${myProduct} transaction: ${myTransaction}`, );
}

export async function get_products() {
  return store.products;
}
