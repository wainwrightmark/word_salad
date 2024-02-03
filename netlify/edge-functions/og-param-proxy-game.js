export default async (request, context) => {
  const url = new URL(request.url);

  // Get the page content.
  const response = await context.next();
  let page = await response.text();

  try {
    const game = url.pathname.substring(6);

    page = page.replace(
      `https://wordsalad.online/images/og_image.png`,
      `https://wordsalad.online/.netlify/functions/image?game=${game}&width=512&height=512`
    );

    page = page.replace(
      `<meta property="og:url" content="https://wordsalad.online/" >`,
      `<meta property="og:url" content="https://wordsalad.online/game/${game}" >`
    );

    return new Response(page, response);
  } catch {
    return response;
  }
};
