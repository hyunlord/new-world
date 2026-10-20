# Vendored from github.com/Valentyn1997/CausalTransformer @ c49a24faa57af966501e241f57a26b528f874a53,
# src/models/utils.py (MIT License, see LICENSE). Kept: grad_reverse, bce, BRTreatmentOutcomeHead, unchanged; dropped:
# the PyTorch-Lightning / MLflow imports and classes (FilteringMlFlowLogger, ROutcomeVitalsHead, AlphaRise,
# clip_normalize_stabilized_weights). See SOURCE.md.
import torch
from torch import nn
import numpy as np
from torch.autograd import Function
import torch.nn.functional as F


def grad_reverse(x, scale=1.0):

    class ReverseGrad(Function):
        """
        Gradient reversal layer
        """

        @staticmethod
        def forward(ctx, x):
            return x

        @staticmethod
        def backward(ctx, grad_output):
            return scale * grad_output.neg()

    return ReverseGrad.apply(x)


def bce(treatment_pred, current_treatments, mode, weights=None):
    if mode == 'multiclass':
        return F.cross_entropy(treatment_pred.permute(0, 2, 1), current_treatments.permute(0, 2, 1), reduce=False, weight=weights)
    elif mode == 'multilabel':
        return F.binary_cross_entropy_with_logits(treatment_pred, current_treatments, reduce=False, weight=weights).mean(dim=-1)
    else:
        raise NotImplementedError()


class BRTreatmentOutcomeHead(nn.Module):
    """Used by CRN, EDCT, MultiInputTransformer"""

    def __init__(self, seq_hidden_units, br_size, fc_hidden_units, dim_treatments, dim_outcome, alpha=0.0, update_alpha=True,
                 balancing='grad_reverse'):
        super().__init__()

        self.seq_hidden_units = seq_hidden_units
        self.br_size = br_size
        self.fc_hidden_units = fc_hidden_units
        self.dim_treatments = dim_treatments
        self.dim_outcome = dim_outcome
        self.alpha = alpha if not update_alpha else 0.0
        self.alpha_max = alpha
        self.balancing = balancing

        self.linear1 = nn.Linear(self.seq_hidden_units, self.br_size)
        self.elu1 = nn.ELU()

        self.linear2 = nn.Linear(self.br_size, self.fc_hidden_units)
        self.elu2 = nn.ELU()
        self.linear3 = nn.Linear(self.fc_hidden_units, self.dim_treatments)

        self.linear4 = nn.Linear(self.br_size + self.dim_treatments, self.fc_hidden_units)
        self.elu3 = nn.ELU()
        self.linear5 = nn.Linear(self.fc_hidden_units, self.dim_outcome)

        self.treatment_head_params = ['linear2', 'linear3']

    def build_treatment(self, br, detached=False):
        if detached:
            br = br.detach()

        if self.balancing == 'grad_reverse':
            br = grad_reverse(br, self.alpha)

        br = self.elu2(self.linear2(br))
        treatment = self.linear3(br)  # Softmax is encapsulated into F.cross_entropy()
        return treatment

    def build_outcome(self, br, current_treatment):
        x = torch.cat((br, current_treatment), dim=-1)
        x = self.elu3(self.linear4(x))
        outcome = self.linear5(x)
        return outcome

    def build_br(self, seq_output):
        br = self.elu1(self.linear1(seq_output))
        return br
