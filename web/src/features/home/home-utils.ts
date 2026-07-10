import type { HomeFeedSection } from '@/lib/api/home'

export function homeSectionId(section: HomeFeedSection) {
  return `home-section-${section.id}`
}

export function scrollToHomeSection(sectionId: string) {
  document.getElementById(sectionId)?.scrollIntoView({
    behavior: 'smooth',
    block: 'start'
  })
}
