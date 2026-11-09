declare const SwaggerUIBundle: {
  (options: { url: string; dom_id: string; deepLinking: boolean; presets: unknown[]; layout: string }): unknown;
  presets: { apis: unknown };
};

window.onload = () => {
  SwaggerUIBundle({
    url: '/api/swagger/openapi.json',
    dom_id: '#swagger-ui',
    deepLinking: true,
    presets: [SwaggerUIBundle.presets.apis],
    layout: 'BaseLayout',
  });
};
