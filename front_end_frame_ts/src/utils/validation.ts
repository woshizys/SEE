/**
 * Validate Email better.
 * @param email 
 */
export const validateEmail = (email: string) => {
  // const reg = /^([a-zA-Z0-9]+[-_\.]?)+@[a-zA-Z0-9]+\.[a-z]+$/;
  const reg = /^([a-zA-Z0-9]+[-_\.]?)+@([a-zA-Z0-9]+\.)+[a-z]+$/;
  return reg.test(email);
};
