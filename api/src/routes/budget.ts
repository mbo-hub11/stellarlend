import { Router, Request, Response, NextFunction } from 'express';
import logger from '../utils/logger';

const router: Router = Router();

interface BudgetPlan {
  lender: string;
  total_budget: number;
  risk_appetite: string;
  allocated_deposit: number;
  allocated_reserve: number;
  expected_apy_bps: number;
  projected_yield: number;
  recommendations: string[];
  created_at: number;
}

interface YieldProjection {
  period_days: number;
  conservative_apy_bps: number;
  moderate_apy_bps: number;
  aggressive_apy_bps: number;
  projected_return_conservative: number;
  projected_return_moderate: number;
  projected_return_aggressive: number;
  assumptions: string;
}

interface AllocationStrategy {
  strategy_name: string;
  deposit_pct_bps: number;
  reserve_pct_bps: number;
  expected_apy_bps: number;
  risk_level: string;
  description: string;
}

const budgetPlans: Map<string, BudgetPlan> = new Map();
const BASIS_POINTS = 10000;

function getAllocation(riskAppetite: string): { depositPct: number; reservePct: number; baseApy: number } {
  if (riskAppetite === 'conservative') return { depositPct: 6000, reservePct: 4000, baseApy: 300 };
  if (riskAppetite === 'aggressive') return { depositPct: 9000, reservePct: 1000, baseApy: 800 };
  return { depositPct: 7500, reservePct: 2500, baseApy: 500 };
}

const budgetController = {
  createPlan(req: Request, res: Response, next: NextFunction) {
    try {
      const { lender, total_budget, risk_appetite } = req.body;
      if (!lender || !total_budget || !risk_appetite) {
        return res.status(400).json({ success: false, error: 'lender, total_budget, and risk_appetite required' });
      }
      const budget = Number(total_budget);
      if (budget <= 0) {
        return res.status(400).json({ success: false, error: 'total_budget must be positive' });
      }
      const { depositPct, reservePct, baseApy } = getAllocation(risk_appetite);
      const allocatedDeposit = (budget * depositPct) / BASIS_POINTS;
      const allocatedReserve = budget - allocatedDeposit;
      const projectedYield = (allocatedDeposit * baseApy) / BASIS_POINTS;
      const recommendations: string[] = [];
      if (risk_appetite === 'conservative') {
        recommendations.push('Maintain higher reserve for safety');
        recommendations.push('Consider compounding yields weekly');
      } else if (risk_appetite === 'aggressive') {
        recommendations.push('Monitor utilization rates closely');
        recommendations.push('Set stop-loss thresholds');
      } else {
        recommendations.push('Rebalance monthly based on market conditions');
      }
      recommendations.push('Diversify across multiple pools');
      const plan: BudgetPlan = {
        lender,
        total_budget: budget,
        risk_appetite,
        allocated_deposit: allocatedDeposit,
        allocated_reserve: allocatedReserve,
        expected_apy_bps: baseApy,
        projected_yield: projectedYield,
        recommendations,
        created_at: Date.now(),
      };
      budgetPlans.set(lender, plan);
      res.json({ success: true, data: plan });
    } catch (error) {
      next(error);
    }
  },

  getPlan(req: Request, res: Response, next: NextFunction) {
    try {
      const { lender } = req.query;
      if (!lender || typeof lender !== 'string') {
        return res.status(400).json({ success: false, error: 'lender query param required' });
      }
      const plan = budgetPlans.get(lender);
      if (!plan) {
        return res.status(404).json({ success: false, error: 'No budget plan found' });
      }
      res.json({ success: true, data: plan });
    } catch (error) {
      next(error);
    }
  },

  projectYields(req: Request, res: Response, next: NextFunction) {
    try {
      const { total_budget, risk_appetite, period_days } = req.body;
      if (!total_budget || !risk_appetite || !period_days) {
        return res.status(400).json({ success: false, error: 'total_budget, risk_appetite, and period_days required' });
      }
      const budget = Number(total_budget);
      const days = Number(period_days);
      if (budget <= 0 || days <= 0) {
        return res.status(400).json({ success: false, error: 'Values must be positive' });
      }
      let conservativeApy: number, moderateApy: number, aggressiveApy: number;
      if (risk_appetite === 'conservative') {
        conservativeApy = 250;
        moderateApy = 350;
        aggressiveApy = 500;
      } else if (risk_appetite === 'aggressive') {
        conservativeApy = 500;
        moderateApy = 800;
        aggressiveApy = 1200;
      } else {
        conservativeApy = 350;
        moderateApy = 500;
        aggressiveApy = 700;
      }
      const periods = days;
      const projection: YieldProjection = {
        period_days: days,
        conservative_apy_bps: conservativeApy,
        moderate_apy_bps: moderateApy,
        aggressive_apy_bps: aggressiveApy,
        projected_return_conservative: (budget * conservativeApy * periods) / (BASIS_POINTS * 365),
        projected_return_moderate: (budget * moderateApy * periods) / (BASIS_POINTS * 365),
        projected_return_aggressive: (budget * aggressiveApy * periods) / (BASIS_POINTS * 365),
        assumptions: 'Based on current protocol utilization and market conditions',
      };
      res.json({ success: true, data: projection });
    } catch (error) {
      next(error);
    }
  },

  getAllocationStrategies(_req: Request, res: Response, next: NextFunction) {
    try {
      const strategies: AllocationStrategy[] = [
        {
          strategy_name: 'conservative',
          deposit_pct_bps: 6000,
          reserve_pct_bps: 4000,
          expected_apy_bps: 300,
          risk_level: 'low',
          description: 'Steady income with minimal risk',
        },
        {
          strategy_name: 'moderate',
          deposit_pct_bps: 7500,
          reserve_pct_bps: 2500,
          expected_apy_bps: 500,
          risk_level: 'medium',
          description: 'Balanced yield and safety',
        },
        {
          strategy_name: 'aggressive',
          deposit_pct_bps: 9000,
          reserve_pct_bps: 1000,
          expected_apy_bps: 800,
          risk_level: 'high',
          description: 'Maximize yield, accept higher risk',
        },
      ];
      res.json({ success: true, data: strategies });
    } catch (error) {
      next(error);
    }
  },

  compareScenarios(req: Request, res: Response, next: NextFunction) {
    try {
      const { total_budget } = req.body;
      if (!total_budget || Number(total_budget) <= 0) {
        return res.status(400).json({ success: false, error: 'total_budget must be positive' });
      }
      const budget = Number(total_budget);
      const scenarios: BudgetPlan[] = ['conservative', 'moderate', 'aggressive'].map((risk) => {
        const { depositPct, reservePct, baseApy } = getAllocation(risk);
        const allocatedDeposit = (budget * depositPct) / BASIS_POINTS;
        const allocatedReserve = budget - allocatedDeposit;
        return {
          lender: 'comparison',
          total_budget: budget,
          risk_appetite: risk,
          allocated_deposit: allocatedDeposit,
          allocated_reserve: allocatedReserve,
          expected_apy_bps: baseApy,
          projected_yield: (allocatedDeposit * baseApy) / BASIS_POINTS,
          recommendations: [],
          created_at: Date.now(),
        };
      });
      res.json({ success: true, data: scenarios });
    } catch (error) {
      next(error);
    }
  },

  calculateRiskAdjusted(req: Request, res: Response, next: NextFunction) {
    try {
      const { allocated_amount, expected_apy_bps, risk_level } = req.body;
      if (!allocated_amount || !expected_apy_bps || !risk_level) {
        return res.status(400).json({ success: false, error: 'allocated_amount, expected_apy_bps, and risk_level required' });
      }
      const amount = Number(allocated_amount);
      const apy = Number(expected_apy_bps);
      let riskFactor: number;
      if (risk_level === 'low') riskFactor = 1;
      else if (risk_level === 'high') riskFactor = 3;
      else riskFactor = 2;
      const annualReturn = (amount * apy) / BASIS_POINTS;
      const riskAdjusted = annualReturn / (riskFactor * 100);
      res.json({ success: true, data: { annual_return: annualReturn, risk_adjusted_return: riskAdjusted, risk_factor: riskFactor } });
    } catch (error) {
      next(error);
    }
  },
};

router.post('/create-plan', budgetController.createPlan);
router.get('/plan', budgetController.getPlan);
router.post('/project-yields', budgetController.projectYields);
router.get('/strategies', budgetController.getAllocationStrategies);
router.post('/compare', budgetController.compareScenarios);
router.post('/risk-adjusted', budgetController.calculateRiskAdjusted);

export default router;
