export function setOptionValues(options, object: any) {
  for (const key of Object.keys(object)) {
    if (options[key] !== undefined) {
      object[key] = options[key];
    }
  }
  return object;
}
