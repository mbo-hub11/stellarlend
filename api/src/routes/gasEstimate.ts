import { Router, Request, Response, NextFunction } from 'express';
import logger from '../utils/logger';

const router: Router = Router();

interface GasEstimate {
  operation: string;
  estimated_instructions: number;
  estimated_cost_xlm: number;
  confidence: number;
  optimization_suggestions: string[];
}

interface OptimizationSuggestion {
  category: string;
  title: string;
  description: string;
  potential_savings_pct: number;
  priority: 'low' | 'medium' | 'high';
}

interface GasComparison {
  operation: string;
  current_estimate: number;
  batched_estimate: number;
  savings_pct: number;
  recommendation: string;
}

const GAS_BASELINES: Record<string, number> = {
  deposit: 10000,
  borrow: 15000,
  repay: 12000,
  withdraw: 11000,
  liquidation: 25000,
  flash_loan: 18000,
  harvest_yield: 8000,
  rebalance: 9000,
  claim_rewards: 7000,
  stake: 8500,
  unstake: 8500,
};

const XLM_PER_INSTRUCTION = 0.000001;

const gasController = {
  estimateGas(req: Request, res: Response, next: NextFunction) {
    try {
      const { operation, amount, include_optimizations } = req.body;
      if (!operation) {
        return res.status(400).json({ success: false, error: 'operation required' });
      }
      const baseline = GAS_BASELINES[operation];
      if (!baseline) {
        return res.status(400).json({ success: false, error: `Unknown operation: ${operation}` });
      }
      const amountMultiplier = amount ? Math.min(Math.max(Number(amount) / 1_000_000, 0.5), 5) : 1;
      const estimatedInstructions = Math.round(baseline * amountMultiplier);
      const estimatedCostXlm = estimatedInstructions * XLM_PER_INSTRUCTION;
      const suggestions: string[] = [];
      if (include_optimizations !== false) {
        if (['deposit', 'borrow', 'repay', 'withdraw'].includes(operation)) {
          suggestions.push('Batch multiple operations in a single transaction to save ~40% gas');
        }
        if (operation === 'liquidation') {
          suggestions.push('Use flash loans for collateral conversion to reduce steps');
          suggestions.push('Liquidate during low network congestion for lower fees');
        }
        if (operation === 'flash_loan') {
          suggestions.push('Minimize callback logic to reduce instruction count');
        }
        suggestions.push('Execute during off-peak hours for lower congestion');
        suggestions.push('Use claim batching for reward accumulation');
      }
      const estimate: GasEstimate = {
        operation,
        estimated_instructions: estimatedInstructions,
        estimated_cost_xlm: estimatedCostXlm,
        confidence: 0.85,
        optimization_suggestions: suggestions,
      };
      res.json({ success: true, data: estimate });
    } catch (error) {
      next(error);
    }
  },

  batchEstimate(req: Request, res: Response, next: NextFunction) {
    try {
      const { operations } = req.body;
      if (!operations || !Array.isArray(operations) || operations.length === 0) {
        return res.status(400).json({ success: false, error: 'operations array required' });
      }
      let totalIndividual = 0;
      const estimates: GasEstimate[] = [];
      for (const op of operations) {
        const baseline = GAS_BASELINES[op.operation] || 10000;
        totalIndividual += baseline;
        estimates.push({
          operation: op.operation,
          estimated_instructions: baseline,
          estimated_cost_xlm: baseline * XLM_PER_INSTRUCTION,
          confidence: 0.85,
          optimization_suggestions: [],
        });
      }
      const batchCost = Math.round(totalIndividual * 0.6);
      const savings = totalIndividual - batchCost;
      const savingsPct = Math.round((savings / totalIndividual) * 100);
      res.json({
        success: true,
        data: {
          individual_estimates: estimates,
          total_individual_cost: totalIndividual,
          batch_cost: batchCost,
          savings,
          savings_pct: savingsPct,
          recommendation: savingsPct > 20
            ? 'Strongly recommend batching operations'
            : 'Consider batching for moderate savings',
        },
      });
    } catch (error) {
      next(error);
    }
  },

  getOptimizationSuggestions(req: Request, res: Response, next: NextFunction) {
    try {
      const { operation } = req.query;
      const suggestions: OptimizationSuggestion[] = [
        {
          category: 'batching',
          title: 'Batch Operations',
          description: 'Combine multiple operations into a single transaction to amortize base costs',
          potential_savings_pct: 40,
          priority: 'high',
        },
        {
          category: 'timing',
          title: 'Off-Peak Execution',
          description: 'Execute transactions during low network congestion periods',
          potential_savings_pct: 15,
          priority: 'medium',
        },
        {
          category: 'storage',
          title: 'Minimize Storage Writes',
          description: 'Reduce the number of storage writes by consolidating state updates',
          potential_savings_pct: 20,
          priority: 'high',
        },
        {
          category: 'compute',
          title: 'Optimize Computation',
          description: 'Pre-compute values off-chain and pass as parameters',
          potential_savings_pct: 10,
          priority: 'medium',
        },
        {
          category: 'approval',
          title: 'Use Unlimited Approvals',
          description: 'Approve maximum token amount once instead of per-transaction',
          potential_savings_pct: 5,
          priority: 'low',
        },
      ];
      const filtered = operation
        ? suggestions.filter(s => s.category === operation || operation === 'all')
        : suggestions;
      res.json({ success: true, data: filtered });
    } catch (error) {
      next(error);
    }
  },

  compareOperations(_req: Request, res: Response, next: NextFunction) {
    try {
      const comparisons: GasComparison[] = Object.entries(GAS_BASELINES).map(([op, cost]) => {
        const batched = Math.round(cost * 0.6);
        const savings = cost - batched;
        return {
          operation: op,
          current_estimate: cost,
          batched_estimate: batched,
          savings_pct: Math.round((savings / cost) * 100),
          recommendation: savings > 3000 ? 'High benefit from batching' : 'Moderate benefit from batching',
        };
      });
      res.json({ success: true, data: comparisons });
    } catch (error) {
      next(error);
    }
  },

  getGasBaselines(_req: Request, res: Response, next: NextFunction) {
    try {
      const baselines = Object.entries(GAS_BASELINES).map(([op, instructions]) => ({
        operation: op,
        instructions,
        cost_xlm: instructions * XLM_PER_INSTRUCTION,
      }));
      res.json({ success: true, data: baselines });
    } catch (error) {
      next(error);
    }
  },

  getTimingRecommendations(_req: Request, res: Response, next: NextFunction) {
    try {
      res.json({
        success: true,
        data: {
          optimal_hours_utc: [2, 3, 4, 5, 6],
          peak_hours_utc: [13, 14, 15, 16, 17, 18],
          recommendation: 'Execute transactions between 2AM-6AM UTC for lowest gas costs',
          estimated_savings_during_off_peak_pct: 25,
        },
      });
    } catch (error) {
      next(error);
    }
  },
};

router.post('/estimate', gasController.estimateGas);
router.post('/batch-estimate', gasController.batchEstimate);
router.get('/optimizations', gasController.getOptimizationSuggestions);
router.get('/compare', gasController.compareOperations);
router.get('/baselines', gasController.getGasBaselines);
router.get('/timing', gasController.getTimingRecommendations);

export default router;
