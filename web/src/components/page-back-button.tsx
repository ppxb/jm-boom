import { useRouter } from '@tanstack/react-router'
import { ArrowLeftIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'

export function PageBackButton() {
  const router = useRouter()

  return (
    <Button
      type="button"
      variant="ghost"
      size="default"
      className="h-11 gap-2 self-start px-3 text-base"
      onClick={() => router.history.back()}
    >
      <ArrowLeftIcon className="size-5" />
      返回
    </Button>
  )
}
