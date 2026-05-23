export async function postControl<TBody extends object>(
  path: string,
  body: TBody,
): Promise<void> {
  const response = await fetch(path, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    throw new Error(`POST ${path} failed with ${response.status}`);
  }
}

