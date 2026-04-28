import React, { useEffect, useLayoutEffect, useState, useCallback, useRef } from 'react';
import { useTutorialStore, TutorialVariant } from '../../stores/tutorialStore';
import {
  SIMPLE_STEPS,
  FULL_STEPS,
  TutorialStep,
  TutorialPlacement,
} from './tutorialSteps';

const HIGHLIGHT_PADDING = 8;
const TOOLTIP_WIDTH = 380;
const TOOLTIP_GAP = 16;

interface Rect {
  top: number;
  left: number;
  width: number;
  height: number;
}

function findTargetRect(targetId: string): Rect | null {
  const el = document.querySelector<HTMLElement>(`[data-tutorial-id="${targetId}"]`);
  if (!el) return null;
  const r = el.getBoundingClientRect();
  return {
    top: r.top - HIGHLIGHT_PADDING,
    left: r.left - HIGHLIGHT_PADDING,
    width: r.width + HIGHLIGHT_PADDING * 2,
    height: r.height + HIGHLIGHT_PADDING * 2,
  };
}

function computeTooltipPosition(
  rect: Rect,
  placement: TutorialPlacement,
  tooltipHeight: number
): { top: number; left: number; resolved: TutorialPlacement } {
  const vw = window.innerWidth;
  const vh = window.innerHeight;

  const order: TutorialPlacement[] = [placement, 'bottom', 'top', 'right', 'left'];
  const seen = new Set<TutorialPlacement>();

  for (const p of order) {
    if (seen.has(p) || p === 'center') continue;
    seen.add(p);

    let top = 0;
    let left = 0;
    switch (p) {
      case 'bottom':
        top = rect.top + rect.height + TOOLTIP_GAP;
        left = rect.left;
        break;
      case 'top':
        top = rect.top - tooltipHeight - TOOLTIP_GAP;
        left = rect.left;
        break;
      case 'right':
        top = rect.top;
        left = rect.left + rect.width + TOOLTIP_GAP;
        break;
      case 'left':
        top = rect.top;
        left = rect.left - TOOLTIP_WIDTH - TOOLTIP_GAP;
        break;
    }

    left = Math.max(16, Math.min(left, vw - TOOLTIP_WIDTH - 16));

    const fits =
      top >= 16 &&
      top + tooltipHeight <= vh - 16 &&
      left >= 0 &&
      left + TOOLTIP_WIDTH <= vw;

    if (fits) {
      return { top, left, resolved: p };
    }
  }

  return {
    top: Math.max(16, (vh - tooltipHeight) / 2),
    left: Math.max(16, (vw - TOOLTIP_WIDTH) / 2),
    resolved: 'center',
  };
}

interface StepCardProps {
  title: string;
  body: string;
  stepLabel: string;
  primaryLabel: string;
  onPrimary: () => void;
  secondaryLabel?: string;
  onSecondary?: () => void;
  skipLabel: string;
  onSkip: () => void;
  style?: React.CSSProperties;
  cardRef?: React.RefObject<HTMLDivElement>;
}

const StepCard: React.FC<StepCardProps> = ({
  title,
  body,
  stepLabel,
  primaryLabel,
  onPrimary,
  secondaryLabel,
  onSecondary,
  skipLabel,
  onSkip,
  style,
  cardRef,
}) => (
  <div
    ref={cardRef}
    role="dialog"
    aria-labelledby="tutorial-title"
    className="fixed bg-white rounded-lg shadow-2xl border border-gray-200"
    style={{
      width: TOOLTIP_WIDTH,
      zIndex: 60,
      ...style,
    }}
  >
    <div className="p-5">
      <div className="text-xs font-medium text-blue-600 uppercase tracking-wide mb-2">
        {stepLabel}
      </div>
      <h3 id="tutorial-title" className="text-lg font-semibold text-gray-900 mb-2">
        {title}
      </h3>
      <p className="text-sm text-gray-600 leading-relaxed">{body}</p>
    </div>
    <div className="flex items-center justify-between px-5 py-3 border-t bg-gray-50 rounded-b-lg">
      <button
        onClick={onSkip}
        className="text-sm text-gray-500 hover:text-gray-700 transition-colors"
      >
        {skipLabel}
      </button>
      <div className="flex items-center gap-2">
        {secondaryLabel && onSecondary && (
          <button
            onClick={onSecondary}
            className="px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-200 rounded transition-colors"
          >
            {secondaryLabel}
          </button>
        )}
        <button
          onClick={onPrimary}
          className="px-4 py-1.5 text-sm bg-blue-600 text-white hover:bg-blue-700 rounded transition-colors"
        >
          {primaryLabel}
        </button>
      </div>
    </div>
  </div>
);

interface VariantChooserCardProps {
  onChoose: (variant: TutorialVariant) => void;
  onSkip: () => void;
  cardRef?: React.RefObject<HTMLDivElement>;
}

const VariantChooserCard: React.FC<VariantChooserCardProps> = ({
  onChoose,
  onSkip,
  cardRef,
}) => (
  <div
    ref={cardRef}
    role="dialog"
    aria-labelledby="tutorial-title"
    className="fixed bg-white rounded-lg shadow-2xl border border-gray-200"
    style={{ width: TOOLTIP_WIDTH + 60, zIndex: 60, position: 'relative' }}
  >
    <div className="p-6">
      <div className="text-xs font-medium text-blue-600 uppercase tracking-wide mb-2">
        Welcome
      </div>
      <h3 id="tutorial-title" className="text-xl font-semibold text-gray-900 mb-2">
        Energy System Simulator
      </h3>
      <p className="text-sm text-gray-600 leading-relaxed mb-5">
        This dashboard simulates a regional power grid hour-by-hour for a year.
        Pick a tour:
      </p>
      <div className="space-y-3">
        <button
          onClick={() => onChoose('simple')}
          className="w-full text-left p-4 border border-gray-200 rounded-lg hover:border-blue-500 hover:bg-blue-50 transition-colors group"
        >
          <div className="flex items-center justify-between mb-1">
            <span className="font-semibold text-gray-900 group-hover:text-blue-700">
              Quick tour
            </span>
            <span className="text-xs text-gray-500">5 steps · ~1 min</span>
          </div>
          <p className="text-sm text-gray-600">
            Capacities, metrics, charts, and the optimizer button. Just the
            essentials.
          </p>
        </button>
        <button
          onClick={() => onChoose('full')}
          className="w-full text-left p-4 border border-gray-200 rounded-lg hover:border-blue-500 hover:bg-blue-50 transition-colors group"
        >
          <div className="flex items-center justify-between mb-1">
            <span className="font-semibold text-gray-900 group-hover:text-blue-700">
              Full tour
            </span>
            <span className="text-xs text-gray-500">11 steps · ~3 min</span>
          </div>
          <p className="text-sm text-gray-600">
            Every control plus a guided look inside the Metrics, Settings, and
            Optimizer menus.
          </p>
        </button>
      </div>
    </div>
    <div className="flex items-center justify-end px-6 py-3 border-t bg-gray-50 rounded-b-lg">
      <button
        onClick={onSkip}
        className="text-sm text-gray-500 hover:text-gray-700 transition-colors"
      >
        Skip tour
      </button>
    </div>
  </div>
);

export const TutorialOverlay: React.FC = () => {
  const isOpen = useTutorialStore((s) => s.isOpen);
  const currentStep = useTutorialStore((s) => s.currentStep);
  const variant = useTutorialStore((s) => s.variant);
  const startVariant = useTutorialStore((s) => s.startVariant);
  const nextStep = useTutorialStore((s) => s.nextStep);
  const prevStep = useTutorialStore((s) => s.prevStep);
  const closeTutorial = useTutorialStore((s) => s.closeTutorial);
  const runAction = useTutorialStore((s) => s.runAction);

  const [rect, setRect] = useState<Rect | null>(null);
  const [tooltipPos, setTooltipPos] = useState<{
    top: number;
    left: number;
    resolved: TutorialPlacement;
  } | null>(null);
  const cardRef = useRef<HTMLDivElement>(null);
  const lastStepRef = useRef<TutorialStep | null>(null);

  const steps: TutorialStep[] | null = variant === 'simple' ? SIMPLE_STEPS : variant === 'full' ? FULL_STEPS : null;
  const step = steps ? steps[currentStep] : null;
  const totalSteps = steps?.length ?? 0;
  const isLast = step != null && currentStep === totalSteps - 1;
  const stepMode = step?.mode ?? (step?.targetId ? 'spotlight' : 'centered');

  // Run onEnter/onExit hooks on step transitions
  useEffect(() => {
    if (!isOpen) {
      // Tour closed — run last step's onExit if any
      if (lastStepRef.current?.onExit) {
        runAction(lastStepRef.current.onExit);
      }
      lastStepRef.current = null;
      return;
    }

    const last = lastStepRef.current;
    if (last && last !== step && last.onExit) {
      runAction(last.onExit);
    }
    if (step && step !== last && step.onEnter) {
      // Defer onEnter slightly so the modal can open and mount before measuring
      runAction(step.onEnter);
    }
    lastStepRef.current = step;
  }, [isOpen, step, runAction]);

  const recomputePositions = useCallback(() => {
    if (!step || stepMode !== 'spotlight') {
      setRect(null);
      setTooltipPos(null);
      return;
    }

    if (!step.targetId) {
      setRect(null);
      setTooltipPos(null);
      return;
    }

    const r = findTargetRect(step.targetId);
    if (!r) {
      setRect(null);
      setTooltipPos(null);
      return;
    }
    setRect(r);

    const cardHeight = cardRef.current?.offsetHeight ?? 200;
    const pos = computeTooltipPosition(r, step.placement ?? 'bottom', cardHeight);
    setTooltipPos(pos);
  }, [step, stepMode]);

  useLayoutEffect(() => {
    if (!isOpen) return;
    // Defer one frame so any onEnter-opened modal can mount before we measure
    const id = requestAnimationFrame(() => recomputePositions());
    return () => cancelAnimationFrame(id);
  }, [isOpen, currentStep, variant, recomputePositions]);

  useEffect(() => {
    if (!isOpen) return;
    const handler = () => recomputePositions();
    window.addEventListener('resize', handler);
    window.addEventListener('scroll', handler, true);
    return () => {
      window.removeEventListener('resize', handler);
      window.removeEventListener('scroll', handler, true);
    };
  }, [isOpen, recomputePositions]);

  // Lock body scroll while open
  useEffect(() => {
    if (!isOpen) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => {
      document.body.style.overflow = prev;
    };
  }, [isOpen]);

  // Keyboard: Escape closes, arrows navigate
  useEffect(() => {
    if (!isOpen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        closeTutorial();
      } else if (e.key === 'ArrowRight') {
        if (variant && currentStep >= totalSteps - 1) {
          closeTutorial();
        } else if (variant) {
          nextStep();
        }
      } else if (e.key === 'ArrowLeft') {
        if (variant) prevStep();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [isOpen, variant, currentStep, totalSteps, nextStep, prevStep, closeTutorial]);

  if (!isOpen) return null;

  // ============ Variant chooser ============
  if (!variant) {
    return (
      <div className="fixed inset-0 z-50">
        <div
          className="absolute inset-0 bg-black/60"
          onClick={closeTutorial}
          aria-hidden
        />
        <div className="relative w-full h-full flex items-center justify-center pointer-events-none">
          <div className="pointer-events-auto">
            <VariantChooserCard
              onChoose={startVariant}
              onSkip={closeTutorial}
              cardRef={cardRef}
            />
          </div>
        </div>
      </div>
    );
  }

  if (!step) return null;

  const handleNext = () => {
    if (isLast) {
      closeTutorial();
    } else {
      nextStep();
    }
  };

  const stepLabel = `Step ${currentStep + 1} of ${totalSteps}`;
  const primaryLabel = isLast ? 'Finish' : 'Next';
  // Back from step 0 goes to chooser
  const secondaryLabel = 'Back';

  // ============ Centered card (no target / explicit centered mode) ============
  if (stepMode === 'centered' || (stepMode === 'spotlight' && !rect)) {
    return (
      <div className="fixed inset-0 z-50">
        <div
          className="absolute inset-0 bg-black/60"
          onClick={closeTutorial}
          aria-hidden
        />
        <div className="relative w-full h-full flex items-center justify-center pointer-events-none">
          <div className="pointer-events-auto">
            <StepCard
              title={step.title}
              body={step.body}
              stepLabel={stepLabel}
              primaryLabel={primaryLabel}
              onPrimary={handleNext}
              secondaryLabel={secondaryLabel}
              onSecondary={prevStep}
              skipLabel={isLast ? 'Close' : 'Skip tour'}
              onSkip={closeTutorial}
              cardRef={cardRef}
              style={{ position: 'relative' }}
            />
          </div>
        </div>
      </div>
    );
  }

  // ============ Modal-overlay step (a real app modal is open behind us) ============
  if (stepMode === 'modal') {
    // No backdrop here — the app's modal already provides one.
    // Pin tutorial card to bottom-center of viewport so it sits below the modal.
    const vh = window.innerHeight;
    const vw = window.innerWidth;
    const cardHeight = cardRef.current?.offsetHeight ?? 220;
    const top = Math.max(16, vh - cardHeight - 24);
    const left = Math.max(16, (vw - TOOLTIP_WIDTH) / 2);
    return (
      <StepCard
        title={step.title}
        body={step.body}
        stepLabel={stepLabel}
        primaryLabel={primaryLabel}
        onPrimary={handleNext}
        secondaryLabel={secondaryLabel}
        onSecondary={prevStep}
        skipLabel={isLast ? 'Close' : 'Skip tour'}
        onSkip={closeTutorial}
        cardRef={cardRef}
        style={{ top, left }}
      />
    );
  }

  // ============ Spotlight step (default) ============
  if (!rect) return null;
  const vw = window.innerWidth;
  const vh = window.innerHeight;

  return (
    <div className="fixed inset-0 z-50">
      <div
        className="absolute bg-black/60"
        style={{ top: 0, left: 0, width: vw, height: rect.top }}
        onClick={closeTutorial}
      />
      <div
        className="absolute bg-black/60"
        style={{
          top: rect.top + rect.height,
          left: 0,
          width: vw,
          height: vh - (rect.top + rect.height),
        }}
        onClick={closeTutorial}
      />
      <div
        className="absolute bg-black/60"
        style={{ top: rect.top, left: 0, width: rect.left, height: rect.height }}
        onClick={closeTutorial}
      />
      <div
        className="absolute bg-black/60"
        style={{
          top: rect.top,
          left: rect.left + rect.width,
          width: vw - (rect.left + rect.width),
          height: rect.height,
        }}
        onClick={closeTutorial}
      />

      <div
        className="absolute rounded-lg pointer-events-none ring-2 ring-yellow-400 animate-pulse"
        style={{
          top: rect.top,
          left: rect.left,
          width: rect.width,
          height: rect.height,
          boxShadow: '0 0 24px rgba(251,188,5,0.6)',
        }}
      />

      {tooltipPos && (
        <StepCard
          title={step.title}
          body={step.body}
          stepLabel={stepLabel}
          primaryLabel={primaryLabel}
          onPrimary={handleNext}
          secondaryLabel={secondaryLabel}
          onSecondary={prevStep}
          skipLabel={isLast ? 'Close' : 'Skip tour'}
          onSkip={closeTutorial}
          cardRef={cardRef}
          style={{ top: tooltipPos.top, left: tooltipPos.left }}
        />
      )}
    </div>
  );
};
