export type FilterOption = {
  label: string
  value: string
  apiValue?: string
}

// Ranking order options
export const RANKING_ORDER_OPTIONS: FilterOption[] = [
  { label: '最新', value: 'new' },
  { label: '最多点赞', value: 'tf' },
  { label: '总排行', value: 'mv' },
  { label: '月排行', value: 'mv_m' },
  { label: '周排行', value: 'mv_w' },
  { label: '日排行', value: 'mv_t' }
]

// Week options
export const WEEK_OPTIONS: FilterOption[] = [
  { label: '周一', value: '1' },
  { label: '周二', value: '2' },
  { label: '周三', value: '3' },
  { label: '周四', value: '4' },
  { label: '周五', value: '5' },
  { label: '周六', value: '6' },
  { label: '周日', value: '7' },
  { label: '已完结', value: '0' }
]

// Week category options
export const WEEK_CATEGORY_OPTIONS: FilterOption[] = [
  { label: '全部', value: 'all' },
  { label: '日漫', value: 'manga' },
  { label: '韩漫', value: 'hanman' }
]

// Ranking category options (dynamic)
export function rankingCategoryOptions(rankTag = ''): FilterOption[] {
  switch (rankTag) {
    case '禁漫汉化组':
      return [{ label: '禁漫汉化组', value: '禁漫汉化组' }]
    case 'hanManTypeMap':
      return [
        { label: '韩漫', value: 'hanman' },
        { label: '韩漫 / 汉化', value: 'hanman_chinese' }
      ]
    case 'qiTaLeiTypeMap':
      return [
        { label: '其他类', value: 'another' },
        { label: '其他类 / 其他漫画', value: 'another_other' },
        { label: '其他类 / 3D', value: 'another_3d' },
        { label: '其他类 / 角色扮演', value: 'another_cosplay' }
      ]
    default:
      return [
        { label: '最新a漫', value: 'latest' },
        { label: '同人', value: 'doujin' },
        { label: '同人 / 汉化', value: 'doujin_chinese' },
        { label: '同人 / 日语', value: 'doujin_japanese' },
        { label: '同人 / CG图集', value: 'doujin_CG' },
        { label: '单本', value: 'single' },
        { label: '单本 / 汉化', value: 'single_chinese' },
        { label: '单本 / 日语', value: 'single_japanese' },
        { label: '单本 / 青年漫', value: 'single_youth' },
        { label: '短篇', value: 'short' },
        { label: '短篇 / 汉化', value: 'short_chinese' },
        { label: '短篇 / 日语', value: 'short_japanese' },
        { label: '其他类', value: 'another' },
        { label: '其他类 / 其他漫画', value: 'another_other' },
        { label: '其他类 / 3D', value: 'another_3d' },
        { label: '其他类 / 角色扮演', value: 'another_cosplay' },
        { label: '韩漫', value: 'hanman' },
        { label: '韩漫 / 汉化', value: 'hanman_chinese' },
        { label: 'English Manga', value: 'meiman' },
        { label: 'English Manga / IRODORI', value: 'meiman_irodori' },
        { label: 'English Manga / FAKKU', value: 'meiman_fakku' },
        { label: 'English Manga / 18scan', value: 'meiman_18scan' },
        { label: 'English Manga / Manhwa', value: 'meiman_manhwa' },
        { label: 'English Manga / Comic', value: 'meiman_comic' },
        { label: 'English Manga / Other', value: 'meiman_other' },
        { label: 'Cosplay', value: 'another_cosplay_direct', apiValue: 'another_cosplay' },
        { label: '3D', value: '3D' },
        { label: '禁漫汉化组', value: '禁漫汉化组' }
      ]
  }
}

export function defaultRankingCategory(rankTag = '') {
  return rankingCategoryOptions(rankTag)[0]?.value ?? 'latest'
}

export function rankingCategoryApiValue(value: string, rankTag = '') {
  return rankingCategoryOptions(rankTag).find(option => option.value === value)?.apiValue ?? value
}
