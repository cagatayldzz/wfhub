export const BASE_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'X-Content-Type-Options': 'nosniff',
};

export const JSON_HEADERS = {
  ...BASE_HEADERS,
  'Content-Type': 'application/json; charset=utf-8',
};
