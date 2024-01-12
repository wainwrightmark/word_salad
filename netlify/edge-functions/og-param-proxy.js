export default async (request, context) => {
  const url = new URL(request.url);

  // Get the page content.
  const response = await context.next();
  let page = await response.text();

  try {
    const daily = url.pathname.substring(7);

    page = page.replace(
      `https://wordsalad.online/images/og_image.png`,
      `https://wordsalad.online/.netlify/functions/image?daily=${daily}&width=512&height=512`
    );

    page = page.replace(
      `<meta content=https://wordsalad.online/ property=og:url>`,
      `<meta content=https://wordsalad.online/daily/${daily} property=og:url>`
    );

    return new Response(page, response);
  } catch {
    return response;
  }
};
