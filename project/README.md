# Project

Monorepo chứa toàn bộ source code, quản lý bằng pnpm workspace.

- `apps/`: các ứng dụng chạy được, mỗi folder con là một project
- `packages/`: thư viện dùng chung giữa các app

Chạy lệnh `pnpm` từ folder này:

```sh
pnpm install                    # cài dependencies cho mọi project
pnpm --filter <tên-package> dev # chạy script của một project
pnpm build                      # build mọi project có script build
pnpm test                       # test mọi project có script test
```
