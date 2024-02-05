export default async (request, context) => {
  const url = new URL(request.url);

  // Get the page content.
  const response = await context.next();
  let page = await response.text();

  try {
    const daily = url.pathname.substring(7);

    page = page.replace(
      `https://wordsalad.online/images/og_image.png`,
      `https://wordsalad.online/.netlify/functions/image?daily=${daily}&width=1080&height=1080`
    );

    page = page.replace(
      `<meta property="og:url" content="https://wordsalad.online/" >`,
      `<meta property="og:url" content="https://wordsalad.online/daily/${daily}" >`
    );

    return new Response(page, response);
  } catch {
    return response;
  }
};
