import { ToArabic } from './to-arabic';

function main(param: string): void {
  const arabic = new ToArabic(param);
  console.log([arabic.normalizeAbsTarget, arabic.isNegative]);
}

main('-弐拾');
