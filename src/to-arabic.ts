import { HYPHENS, PLUSES } from './constants';

export class ToArabic {
  isNegative: boolean;
  normalizeAbsTarget: string;

  constructor(param: string) {
    const data = param.trim();
    if (data === '') {
      throw new Error('Kanji is empty');
    }
    this.isNegative = this.checkIsNegative(data);
    this.normalizeAbsTarget = data;
    const plasesSet = new Set<string>(PLUSES);
    if (this.isNegative || plasesSet.has(data[0])) {
      this.normalizeAbsTarget = data.slice(1);
    }
  }

  private checkIsNegative(data: string): boolean {
    const hyphensSet = new Set<string>(HYPHENS);
    return hyphensSet.has(data[0]);
  }
}
