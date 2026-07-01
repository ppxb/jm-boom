import { useRouter } from '@tanstack/react-router'
import { ArrowLeftIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'

export function PageBackButton() {
  const router = useRouter()

  return (
    <Button
      variant="ghost"
      size="sm"
      className="self-start"
      onClick={() => router.history.back()}
    >
      <ArrowLeftIcon className="size-4" />
      返回
    </Button>
  )
}
