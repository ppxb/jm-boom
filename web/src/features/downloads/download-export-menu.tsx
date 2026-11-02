import { ArchiveIcon, EllipsisIcon, FileTextIcon, LoaderCircleIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import type { DownloadTask } from '@/lib/api/download'
import { startDownloadExport, useDownloadExportState } from './download-export'

export function DownloadExportMenu({ task }: { task: DownloadTask }) {
  const active = useDownloadExportState(state => state.active)

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="rounded-full border border-white/15 bg-black/55 text-white shadow-sm backdrop-blur hover:bg-black/75 hover:text-white dark:hover:bg-black/75"
            disabled={active !== null}
            aria-label="导出下载"
          >
            {active?.taskId === task.taskId ? (
              <LoaderCircleIcon className="animate-spin" />
            ) : (
              <EllipsisIcon />
            )}
          </Button>
        }
      />
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => void startDownloadExport(task, 'pdf')}>
          <FileTextIcon />
          导出 PDF
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => void startDownloadExport(task, 'cbz')}>
          <ArchiveIcon />
          导出 CBZ
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
