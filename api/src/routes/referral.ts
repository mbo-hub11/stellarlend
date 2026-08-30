import { Router, Request, Response, NextFunction } from 'express';
import logger from '../utils/logger';

const router: Router = Router();

interface AffiliateCode {
  owner: string;
  code: string;
  created_at: number;
  total_earned: number;
  total_referrals: number;
  is_active: boolean;
}

interface ReferralRecord {
  referrer: string;
  referee: string;
  registered_at: number;
  total_fees_generated: number;
  referrer_earned: number;
  referee_discount_bps: number;
}

interface AffiliateStats {
  total_referrals: number;
  total_earned: number;
  total_claimed: number;
  claimable: number;
  lifetime_fees_generated: number;
  tier: number;
}

interface GlobalMetrics {
  total_codes_created: number;
  total_referrals: number;
  total_rewards_distributed: number;
  total_fees_generated: number;
  total_referee_discounts: number;
}

const codes: Map<string, AffiliateCode> = new Map();
const ownerCodes: Map<string, string> = new Map();
const referrals: Map<string, ReferralRecord> = new Map();
let globalMetrics: GlobalMetrics = {
  total_codes_created: 0,
  total_referrals: 0,
  total_rewards_distributed: 0,
  total_fees_generated: 0,
  total_referee_discounts: 0,
};

const DEFAULT_REWARD_BPS = 2500;
const DEFAULT_REFEREE_DISCOUNT_BPS = 500;
const BASIS_POINTS = 10000;

function computeTier(referrals: number): number {
  if (referrals >= 50) return 3;
  if (referrals >= 15) return 2;
  if (referrals >= 5) return 1;
  return 0;
}

const referralController = {
  registerCode(req: Request, res: Response, next: NextFunction) {
    try {
      const { userAddress, code } = req.body;
      if (!userAddress || !code) {
        return res.status(400).json({ success: false, error: 'userAddress and code required' });
      }
      if (codes.has(code)) {
        return res.status(400).json({ success: false, error: 'Code already exists' });
      }
      const affiliateCode: AffiliateCode = {
        owner: userAddress,
        code,
        created_at: Date.now(),
        total_earned: 0,
        total_referrals: 0,
        is_active: true,
      };
      codes.set(code, affiliateCode);
      ownerCodes.set(userAddress, code);
      globalMetrics.total_codes_created++;
      res.json({ success: true, data: affiliateCode });
    } catch (error) {
      next(error);
    }
  },

  refer(req: Request, res: Response, next: NextFunction) {
    try {
      const { refereeAddress, code } = req.body;
      if (!refereeAddress || !code) {
        return res.status(400).json({ success: false, error: 'refereeAddress and code required' });
      }
      const affiliateCode = codes.get(code);
      if (!affiliateCode || !affiliateCode.is_active) {
        return res.status(400).json({ success: false, error: 'Invalid or inactive code' });
      }
      if (affiliateCode.owner === refereeAddress) {
        return res.status(400).json({ success: false, error: 'Self-referral not allowed' });
      }
      if (referrals.has(refereeAddress)) {
        return res.status(400).json({ success: false, error: 'Already registered' });
      }
      const record: ReferralRecord = {
        referrer: affiliateCode.owner,
        referee: refereeAddress,
        registered_at: Date.now(),
        total_fees_generated: 0,
        referrer_earned: 0,
        referee_discount_bps: DEFAULT_REFEREE_DISCOUNT_BPS,
      };
      referrals.set(refereeAddress, record);
      affiliateCode.total_referrals++;
      globalMetrics.total_referrals++;
      res.json({ success: true, data: record });
    } catch (error) {
      next(error);
    }
  },

  recordFee(req: Request, res: Response, next: NextFunction) {
    try {
      const { refereeAddress, feeAmount } = req.body;
      if (!refereeAddress || !feeAmount) {
        return res.status(400).json({ success: false, error: 'refereeAddress and feeAmount required' });
      }
      const record = referrals.get(refereeAddress);
      if (!record) {
        return res.status(400).json({ success: false, error: 'Referral not found' });
      }
      const reward = (Number(feeAmount) * DEFAULT_REWARD_BPS) / BASIS_POINTS;
      const discount = (Number(feeAmount) * DEFAULT_REFEREE_DISCOUNT_BPS) / BASIS_POINTS;
      record.total_fees_generated += Number(feeAmount);
      record.referrer_earned += reward;
      const code = ownerCodes.get(record.referrer);
      if (code) {
        const affiliate = codes.get(code);
        if (affiliate) affiliate.total_earned += reward;
      }
      globalMetrics.total_rewards_distributed += reward;
      globalMetrics.total_fees_generated += Number(feeAmount);
      globalMetrics.total_referee_discounts += discount;
      res.json({ success: true, data: { reward, discount } });
    } catch (error) {
      next(error);
    }
  },

  claimRewards(req: Request, res: Response, next: NextFunction) {
    try {
      const { userAddress } = req.body;
      if (!userAddress) {
        return res.status(400).json({ success: false, error: 'userAddress required' });
      }
      const code = ownerCodes.get(userAddress);
      if (!code) {
        return res.status(400).json({ success: false, error: 'No affiliate code found' });
      }
      const affiliate = codes.get(code);
      if (!affiliate || affiliate.total_earned <= 0) {
        return res.status(400).json({ success: false, error: 'Nothing to claim' });
      }
      const claimable = affiliate.total_earned;
      affiliate.total_earned = 0;
      logger.info(`Rewards claimed: ${userAddress} claimed ${claimable}`);
      res.json({ success: true, data: { claimed: claimable } });
    } catch (error) {
      next(error);
    }
  },

  getStats(req: Request, res: Response, next: NextFunction) {
    try {
      const { userAddress } = req.query;
      if (!userAddress || typeof userAddress !== 'string') {
        return res.status(400).json({ success: false, error: 'userAddress query param required' });
      }
      const code = ownerCodes.get(userAddress);
      if (!code) {
        return res.status(404).json({ success: false, error: 'No affiliate code found' });
      }
      const affiliate = codes.get(code);
      if (!affiliate) {
        return res.status(404).json({ success: false, error: 'Affiliate not found' });
      }
      const stats: AffiliateStats = {
        total_referrals: affiliate.total_referrals,
        total_earned: affiliate.total_earned,
        total_claimed: 0,
        claimable: affiliate.total_earned,
        lifetime_fees_generated: 0,
        tier: computeTier(affiliate.total_referrals),
      };
      res.json({ success: true, data: stats });
    } catch (error) {
      next(error);
    }
  },

  getGlobalMetrics(_req: Request, res: Response, next: NextFunction) {
    try {
      res.json({ success: true, data: globalMetrics });
    } catch (error) {
      next(error);
    }
  },

  getReferralRecord(req: Request, res: Response, next: NextFunction) {
    try {
      const { refereeAddress } = req.query;
      if (!refereeAddress || typeof refereeAddress !== 'string') {
        return res.status(400).json({ success: false, error: 'refereeAddress query param required' });
      }
      const record = referrals.get(refereeAddress);
      if (!record) {
        return res.status(404).json({ success: false, error: 'Referral record not found' });
      }
      res.json({ success: true, data: record });
    } catch (error) {
      next(error);
    }
  },
};

router.post('/register-code', referralController.registerCode);
router.post('/refer', referralController.refer);
router.post('/record-fee', referralController.recordFee);
router.post('/claim', referralController.claimRewards);
router.get('/stats', referralController.getStats);
router.get('/metrics', referralController.getGlobalMetrics);
router.get('/record', referralController.getReferralRecord);

export default router;
