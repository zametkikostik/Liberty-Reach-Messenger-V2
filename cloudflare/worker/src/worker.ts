export default {
  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/health") {
      return Response.json({ ok: true, service: "liberty-worker" });
    }
    if (url.pathname === "/push" && request.method === "POST") {
      const body = await request.text();
      return Response.json({ accepted: true, bytes: body.length });
    }
    return new Response("Liberty Messenger Worker V2", { status: 200 });
  },
};
