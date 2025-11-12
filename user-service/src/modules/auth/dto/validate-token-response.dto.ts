import { ApiProperty } from '@nestjs/swagger';

export class ValidateTokenResponseDto {
  @ApiProperty({ description: 'Whether the token is valid' })
  valid: boolean;

  @ApiProperty({ description: 'User ID if token is valid', required: false })
  user_id?: string;

  @ApiProperty({ description: 'User email if token is valid', required: false })
  email?: string;

  @ApiProperty({ description: 'User push token if available', required: false })
  push_token?: string;

  @ApiProperty({ description: 'Token expiration timestamp', required: false })
  expires_at?: number;

  @ApiProperty({ description: 'Reason for invalidity', required: false })
  reason?: string;
}
