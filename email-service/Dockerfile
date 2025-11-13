# FROM node:20-alpine AS builder

# WORKDIR /app

# COPY package*.json ./

# RUN npm ci --fetch-retries 5 --fetch-retry-maxtimeout 600000

# COPY . .

# RUN npm run build

# FROM node:20-alpine

# WORKDIR /app

# COPY package*.json ./

# COPY --from=builder /app/node_modules ./node_modules

# COPY --from=builder /app/dist ./dist

# RUN addgroup -g 1001 -S nodejs && \
#     adduser -S nestjs -u 1001

# USER nestjs

# EXPOSE 3003

# # Health check
# HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
#   CMD /usr/local/bin/node -e "require('http').get('http://localhost:3003/health', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# CMD ["node", "dist/main"]


FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./

RUN npm ci --fetch-retries 5 --fetch-retry-maxtimeout 600000

COPY . .

RUN npm run build

# List the build output for debugging
RUN ls -la dist/ || echo "dist directory not found"

FROM node:20-alpine

WORKDIR /app

COPY package*.json ./

COPY --from=builder /app/node_modules ./node_modules

COPY --from=builder /app/dist ./dist

# List what we copied
RUN ls -la dist/ || echo "dist directory not found"

RUN addgroup -g 1001 -S nodejs && \
    adduser -S nestjs -u 1001

USER nestjs

EXPOSE 3003

HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD /usr/local/bin/node -e "require('http').get('http://localhost:3003/health', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Try to find and run the main file
CMD ["sh", "-c", "node dist/main.js || node dist/main/main.js || node dist/src/main.js"]