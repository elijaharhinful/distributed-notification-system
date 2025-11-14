import {
  Injectable,
  Inject,
  Logger,
  OnModuleInit,
  OnModuleDestroy,
} from '@nestjs/common';
import { ClientProxy } from '@nestjs/microservices';
import { lastValueFrom, timeout } from 'rxjs';

@Injectable()
export class RabbitMQService implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(RabbitMQService.name);

  constructor(
    @Inject('RABBITMQ_CLIENT') private readonly client: ClientProxy,
  ) {}

  async onModuleInit(): Promise<void> {
    try {
      await this.client.connect();
      this.logger.log('✅ Connected to RabbitMQ');
    } catch (error) {
      this.logger.error('❌ Failed to connect to RabbitMQ:', error);
      throw error;
    }
  }
}

  private async setupQueues() {
    const exchange = 'notifications.direct';
    const emailQueue = 'email.queue';
    const failedQueue = 'failed.queue';

    // Declare exchange
    await this.channel.assertExchange(exchange, 'direct', { durable: true });

    // Declare dead letter exchange and queue
    await this.channel.assertExchange('dlx.exchange', 'direct', { durable: true });
    await this.channel.assertQueue(failedQueue, {
      durable: true,
    });
    await this.channel.bindQueue(failedQueue, 'dlx.exchange', 'failed');

    // Declare email queue with DLX
    await this.channel.assertQueue(emailQueue, {
      durable: true,
    });

    // Bind queue to exchange
    await this.channel.bindQueue(emailQueue, exchange, 'email');

    this.logger.log('Queues and exchanges configured');
  }

  async onModuleDestroy(): Promise<void> {
    try {
      await this.client.close();
      this.logger.log('RabbitMQ connection closed');
    } catch (error) {
      this.logger.error('Error closing RabbitMQ connection:', error);
    }
  }

  publishToQueue(queue: string, message: unknown): void {
    try {
      this.client.emit(queue, message);
      this.logger.log(`Message published to queue: ${queue}`);
    } catch (error) {
      this.logger.error(`Failed to publish to ${queue}:`, error);
      throw error;
    }
  }

  async sendWithResponse<T>(
    pattern: string,
    data: unknown,
    timeoutMs: number = 5000,
  ): Promise<T> {
    try {
      const response$ = this.client
        .send<T>(pattern, data)
        .pipe(timeout(timeoutMs));
      return await lastValueFrom(response$);
    } catch (error) {
      this.logger.error(
        `Failed to send message with pattern ${pattern}:`,
        error,
      );
      throw error;
    }
  }

  publish(exchange: string, routingKey: string, message: unknown): void {
    try {
      const pattern = `${exchange}.${routingKey}`;
      this.client.emit(pattern, message);
      this.logger.log(
        `Message published to exchange: ${exchange}, routingKey: ${routingKey}`,
      );
    } catch (error) {
      this.logger.error(`Failed to publish to ${exchange}:`, error);
      throw error;
    }
  }
}
